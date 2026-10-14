mod icon;
mod inhibit;

use icon::Icons;
use inhibit::Inhibitor;
use ksni::TrayMethods;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

enum Command {
    SetActive(bool),
    Quit,
}

struct Tray {
    active: bool,
    icons: Icons,
    tx: UnboundedSender<Command>,
}

impl Tray {
    fn toggle(&mut self) {
        // The icon flips immediately; main() reverts it if D-Bus fails.
        self.active = !self.active;
        let _ = self.tx.send(Command::SetActive(self.active));
    }
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        "Geekoffee".into()
    }

    fn category(&self) -> ksni::Category {
        ksni::Category::SystemServices
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        if self.active {
            self.icons.on.clone()
        } else {
            self.icons.off.clone()
        }
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let description = if self.active {
            "Keeping awake: screen lock and sleep are blocked"
        } else {
            "Inactive: using your normal power settings"
        };
        ksni::ToolTip {
            title: "Geekoffee".into(),
            description: description.into(),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.toggle();
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            CheckmarkItem {
                label: "Keep awake".into(),
                checked: self.active,
                activate: Box::new(|this: &mut Self| this.toggle()),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx.send(Command::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::Connection::session().await?;
    let mut inhibitor = Inhibitor::new(&conn).await?;

    let (tx, mut rx) = unbounded_channel();
    let tray = Tray {
        active: false,
        icons: Icons::load(),
        tx,
    }
    .spawn()
    .await?;

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        tokio::select! {
            cmd = rx.recv() => match cmd {
                Some(Command::SetActive(active)) => {
                    if let Err(e) = inhibitor.set(active).await {
                        eprintln!("geekoffee: failed to change inhibition: {e}");
                    }
                    // Keep the tray in sync with what actually took effect.
                    let actual = inhibitor.is_active();
                    tray.update(|t| t.active = actual).await;
                }
                Some(Command::Quit) | None => break,
            },
            _ = sigterm.recv() => break,
            _ = sigint.recv() => break,
        }
    }

    // Hand control back to the user's power settings.
    if let Err(e) = inhibitor.set(false).await {
        eprintln!("geekoffee: failed to release inhibition: {e}");
    }
    tray.shutdown().await;
    Ok(())
}
