//! Screen-lock and sleep inhibition via the freedesktop D-Bus APIs that
//! KDE's PowerDevil and screen locker implement.
//!
//! Nothing in the user's power settings is modified: an inhibition is a
//! temporary request that is dropped on `UnInhibit`, or automatically by the
//! bus if this process exits or crashes.

use zbus::{Connection, proxy};

const APP_NAME: &str = "Geekoffee";
const REASON: &str = "Keep awake enabled from the tray";

/// Blocks screen locking and screen dimming/blanking.
#[proxy(
    interface = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/org/freedesktop/ScreenSaver"
)]
trait ScreenSaver {
    fn inhibit(&self, application_name: &str, reason: &str) -> zbus::Result<u32>;
    fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
}

/// Blocks automatic suspend/hibernate on idle.
#[proxy(
    interface = "org.freedesktop.PowerManagement.Inhibit",
    default_service = "org.freedesktop.PowerManagement",
    default_path = "/org/freedesktop/PowerManagement/Inhibit"
)]
trait PowerManagement {
    fn inhibit(&self, application: &str, reason: &str) -> zbus::Result<u32>;
    fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
}

struct Cookies {
    screensaver: u32,
    power: u32,
}

pub struct Inhibitor {
    screensaver: ScreenSaverProxy<'static>,
    power: PowerManagementProxy<'static>,
    cookies: Option<Cookies>,
}

impl Inhibitor {
    pub async fn new(conn: &Connection) -> zbus::Result<Self> {
        Ok(Self {
            screensaver: ScreenSaverProxy::new(conn).await?,
            power: PowerManagementProxy::new(conn).await?,
            cookies: None,
        })
    }

    pub fn is_active(&self) -> bool {
        self.cookies.is_some()
    }

    pub async fn set(&mut self, active: bool) -> zbus::Result<()> {
        if active {
            self.enable().await
        } else {
            self.disable().await
        }
    }

    async fn enable(&mut self) -> zbus::Result<()> {
        if self.cookies.is_some() {
            return Ok(());
        }
        let screensaver = self.screensaver.inhibit(APP_NAME, REASON).await?;
        let power = match self.power.inhibit(APP_NAME, REASON).await {
            Ok(cookie) => cookie,
            Err(e) => {
                // Don't leave a half-applied inhibition behind.
                let _ = self.screensaver.un_inhibit(screensaver).await;
                return Err(e);
            }
        };
        self.cookies = Some(Cookies { screensaver, power });
        Ok(())
    }

    async fn disable(&mut self) -> zbus::Result<()> {
        let Some(cookies) = self.cookies.take() else {
            return Ok(());
        };
        // Attempt both even if the first fails.
        let screensaver = self.screensaver.un_inhibit(cookies.screensaver).await;
        let power = self.power.un_inhibit(cookies.power).await;
        screensaver.and(power)
    }
}
