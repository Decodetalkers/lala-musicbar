//! # D-Bus interface proxy for: `org.freedesktop.Notifications`
//!
//! This code was generated by `zbus-xmlgen` `4.1.0` from D-Bus introspection data.
//! Source: `Interface '/org/freedesktop/Notifications' from service 'org.freedesktop.Notifications' on system bus`.
//!
//! Please read the document of [notification-spec](https://specifications.freedesktop.org/notification-spec)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use glob::glob;
use zbus::{interface, object_server::SignalContext, zvariant::OwnedValue};

use std::sync::{Arc, LazyLock, RwLock};

use futures::channel::mpsc::Sender;
use zbus::ConnectionBuilder;

use zbus::zvariant::{SerializeDict, Type};

/// The notification expired
pub const NOTIFICATION_DELETED_BY_EXPIRED: u32 = 1;
/// The notification was dismissed by the user.
pub const NOTIFICATION_DELETED_BY_USER: u32 = 2;

/// The notification was closed by a call to CloseNotification.
pub const NOTIFICATION_CLOSED_BY_DBUS: u32 = 3;

/// Undefined/reserved reasons.
pub const NOTIFICATION_CLOSED_BY_UNKNOWN_REASON: u32 = 4;

static ICON_CACHE: LazyLock<Arc<RwLock<HashMap<String, ImageInfo>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

use std::hash::Hash;

use std::sync::atomic::{self, AtomicU32};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The id of the window.
///
/// Internally Iced reserves `window::Id::MAIN` for the first window spawned.
struct Id(u32);

static COUNT: AtomicU32 = AtomicU32::new(0);

impl Id {
    fn unique() -> Id {
        Id(COUNT.fetch_add(1, atomic::Ordering::Relaxed))
    }
}

/// Describe the image information.
#[derive(Type, Debug, SerializeDict, OwnedValue, Clone)]
struct ImageData {
    width: i32,
    height: i32,
    rowstride: i32,
    has_alpha: bool,
    bits_per_sample: i32,
    channels: i32,
    data: Vec<u8>,
}

/// NotifyMessage about the add and remove
#[derive(Debug, Clone)]
pub enum NotifyMessage {
    UnitAdd(NotifyUnit),
    UnitRemove(u32),
}

fn lazy_get_icon(icon: &str) -> Option<ImageInfo> {
    let icon_cache = ICON_CACHE.read().unwrap();
    if icon_cache.contains_key(icon) {
        return icon_cache.get(icon).cloned();
    }
    drop(icon_cache);
    let mut icon_cache = ICON_CACHE.write().unwrap();
    if let Some(path) = get_svg_icon("hicolor", icon) {
        icon_cache.insert(icon.to_string(), ImageInfo::Svg(path.clone()));
        return Some(ImageInfo::Svg(path));
    }
    if let Some(path) = get_png_icon("hicolor", icon) {
        icon_cache.insert(icon.to_string(), ImageInfo::Png(path.clone()));
        return Some(ImageInfo::Png(path));
    }
    if let Some(path) = get_jpeg_icon("hicolor", icon) {
        icon_cache.insert(icon.to_string(), ImageInfo::Jpg(path.clone()));
        return Some(ImageInfo::Jpg(path));
    }
    if let Some(path) = get_pixmap_icon(icon) {
        icon_cache.insert(icon.to_string(), ImageInfo::Png(path.clone()));
        return Some(ImageInfo::Png(path));
    }
    None
}

fn get_svg_icon(theme: &str, icon: &str) -> Option<PathBuf> {
    glob(&format!("/usr/share/icons/{theme}/**/**/{icon}.svg"))
        .ok()?
        .next()?
        .ok()
}

fn get_png_icon(theme: &str, icon: &str) -> Option<PathBuf> {
    glob(&format!("/usr/share/icons/{theme}/**/**/{icon}.png"))
        .ok()?
        .next()?
        .ok()
}

fn get_pixmap_icon(icon: &str) -> Option<PathBuf> {
    let path = Path::new(format!("/usr/share/pixmaps/{icon}.png").as_str()).to_owned();
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn get_jpeg_icon(theme: &str, icon: &str) -> Option<PathBuf> {
    glob(&format!("/usr/share/icons/{theme}/**/**/{icon}.jpg"))
        .ok()?
        .next()?
        .ok()
}

/// storage the hint of notification
#[derive(Debug, Clone)]
pub struct NotifyHint {
    image_data: Option<ImageData>,
    desktop_entry: Option<String>,
}

/// contain the info about image
#[derive(Debug, Clone)]
pub enum ImageInfo {
    /// raw data of image
    Data {
        width: i32,
        height: i32,
        pixels: Vec<u8>,
    },
    /// svg path
    Svg(PathBuf),
    /// png path
    Png(PathBuf),
    /// jpeg path
    Jpg(PathBuf),
}

impl NotifyHint {
    fn desktop_image(&self) -> Option<ImageInfo> {
        self.desktop_entry
            .as_ref()
            .and_then(|icon| lazy_get_icon(icon))
    }
    fn hint_image(&self) -> Option<ImageInfo> {
        self.image_data.as_ref().map(|data| ImageInfo::Data {
            width: data.width,
            height: data.height,
            pixels: data.data.clone(),
        })
    }
}

/// Describe the information in every time notify send
#[derive(Debug, Clone)]
pub struct NotifyUnit {
    /// application from
    pub app_name: String,
    /// The id of notify
    pub id: u32,
    /// The icon
    pub icon: String,
    /// summery of a notify
    pub summery: String,
    /// the body of notify, use rich text
    pub body: String,
    /// supported actions
    pub actions: Vec<String>,
    /// timeout, if equal to -1, then means it should been always shown
    pub timeout: i32,
    /// other information like image-data
    pub hint: NotifyHint,
}

impl NotifyUnit {
    /// if this notify unit support inline_reply
    pub fn inline_reply_support(&self) -> bool {
        self.actions.contains(&"inline-reply".to_owned())
    }

    /// Get the image inside the unit
    /// It will use the image in hint first
    /// Then use icon from the param by notify
    /// Finally use the application icon
    pub fn image(&self) -> Option<ImageInfo> {
        if let Some(hint_image) = self.hint.hint_image() {
            return Some(hint_image);
        }
        if !self.icon.is_empty() {
            let path = Path::new(&self.icon);
            if path.exists() {
                if self.icon.ends_with("svg") {
                    return Some(ImageInfo::Svg(path.into()));
                } else if self.icon.ends_with("jpg") {
                    return Some(ImageInfo::Jpg(path.into()));
                } else {
                    return Some(ImageInfo::Png(path.into()));
                }
            }
            if let Some(info) = lazy_get_icon(&self.icon) {
                return Some(info);
            }
        }
        self.hint.desktop_image()
    }
}

/// Set the server info in `get_server_information`
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub spec_version: String,
}

/// Do not care this name. it is the Interface name of `org.freedesktop.Notifications`
#[derive(Debug)]
pub struct LaLaMako<T: From<NotifyMessage> + Send> {
    capabilities: Vec<String>,
    sender: Sender<T>,
    version: VersionInfo,
}

#[interface(name = "org.freedesktop.Notifications")]
impl<T: From<NotifyMessage> + Send + 'static> LaLaMako<T> {
    // CloseNotification method
    async fn close_notification(
        &mut self,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
        id: u32,
    ) -> zbus::fdo::Result<()> {
        Self::notification_closed(&ctx, id, NOTIFICATION_DELETED_BY_USER)
            .await
            .ok();
        self.sender
            .try_send(NotifyMessage::UnitRemove(id).into())
            .ok();
        Ok(())
    }

    /// GetCapabilities method
    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }

    /// GetServerInformation method
    fn get_server_information(&self) -> (String, String, String, String) {
        let VersionInfo {
            name,
            vendor,
            version,
            spec_version,
        } = &self.version;
        (
            name.clone(),
            vendor.clone(),
            version.clone(),
            spec_version.clone(),
        )
    }

    // Notify method
    #[allow(clippy::too_many_arguments)]
    fn notify(
        &mut self,
        app_name: &str,
        replaced_id: u32,
        icon: &str,
        summery: &str,
        body: &str,
        actions: Vec<&str>,
        mut hints: std::collections::HashMap<&str, OwnedValue>,
        timeout: i32,
    ) -> zbus::fdo::Result<u32> {
        let id = if replaced_id == 0 {
            Id::unique()
        } else {
            Id(replaced_id)
        };
        let mut image_data: Option<ImageData> =
            hints.remove("image-data").and_then(|v| v.try_into().ok());
        if image_data.is_none() {
            // why send data here...
            image_data = hints.remove("icon_data").and_then(|v| v.try_into().ok());
        }
        let desktop_entry: Option<String> = hints
            .remove("desktop-entry")
            .and_then(|v| v.try_into().ok());
        self.sender
            .try_send(
                NotifyMessage::UnitAdd(NotifyUnit {
                    app_name: app_name.to_string(),
                    id: id.0,
                    icon: icon.to_string(),
                    summery: summery.to_string(),
                    body: body.to_string(),
                    actions: actions.iter().map(|a| a.to_string()).collect(),
                    timeout,
                    hint: NotifyHint {
                        image_data,
                        desktop_entry,
                    },
                })
                .into(),
            )
            .ok();
        Ok(id.0)
    }

    /// Invoke Action
    #[zbus(signal)]
    pub async fn action_invoked(
        ctx: &SignalContext<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;

    /// Notification Reply
    #[zbus(signal)]
    pub async fn notification_replied(
        ctx: &SignalContext<'_>,
        id: u32,
        text: &str,
    ) -> zbus::Result<()>;

    /// NotificationClosed signal
    #[zbus(signal)]
    pub async fn notification_closed(
        ctx: &SignalContext<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;
}

/// org.freedesktop.Notifications server path
pub const NOTIFICATION_SERVICE_PATH: &str = "/org/freedesktop/Notifications";
/// org.freedesktop.Notifications server name
pub const NOTIFICATION_SERVICE_NAME: &str = "/org/freedesktop/Notifications";
/// org.freedesktop.Notifications interface name
pub const NOTIFICATION_SERVICE_INTERFACE: &str = "/org/freedesktop/Notifications";

/// action_invoked signal
pub const ACTION_INVOKED: &str = "action_invoked";

/// notification_closed signal
pub const NOTIFICATION_CLOSED: &str = "notification_closed";

/// default action name
pub const DEFAULT_ACTION: &str = "default";

/// start a connection
pub async fn start_connection<T: From<NotifyMessage> + Send + 'static>(
    sender: Sender<T>,
    capabilities: Vec<String>,
    version: VersionInfo,
) -> Result<zbus::Connection, zbus::Error> {
    ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at(
            "/org/freedesktop/Notifications",
            LaLaMako {
                sender,
                capabilities,
                version,
            },
        )?
        .build()
        .await
}
