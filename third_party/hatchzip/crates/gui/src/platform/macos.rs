use std::path::PathBuf;

pub const ENGINE_NAME: &str = "7zz";
pub fn quiet_command(_command: &mut std::process::Command) {}

/// The settings switch, named after this system's file manager.
pub const SHELL_MENU_LABEL: &str = "Show in the Finder context menu";

/// hatchzip-shell can't add Finder entries yet, so the setting is hidden.
pub const SHELL_MENU: bool = false;

/// `~/Library/Application Support/HatchZip`.
pub fn config_dir() -> Option<PathBuf> {
    Some(std::env::home_dir()?.join("Library/Application Support/7zip-hatch"))
}

// Finder's Open With and document double-clicks arrive as Apple Events,
// including when the app is already running. winit does not forward these.
use objc2::rc::Retained;
use objc2::{AnyThread, define_class, msg_send, sel};
use objc2_foundation::{
    NSAppleEventDescriptor, NSAppleEventManager, NSNotification, NSNotificationCenter, NSObject,
    ns_string,
};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static DOCUMENT_DROPS: RefCell<Option<Rc<crate::backend::Drops>>> = const { RefCell::new(None) };
    static DOCUMENT_HANDLER: RefCell<Option<Retained<OpenDocuments>>> = const { RefCell::new(None) };
}

define_class!(
    #[unsafe(super(NSObject))]
    struct OpenDocuments;

    impl OpenDocuments {
        #[unsafe(method(applicationLaunching:))]
        fn application_launching(&self, _notification: &NSNotification) {
            self.register_events();
        }

        #[unsafe(method(handleDocuments:withReplyEvent:))]
        fn handle_documents(&self, event: &NSAppleEventDescriptor, _reply: &NSAppleEventDescriptor) {
            let Some(list) = event.paramDescriptorForKeyword(u32::from_be_bytes(*b"----")) else { return };
            // This window opens one archive at a time, as the Open dialog does.
            if let Some(item) = list.descriptorAtIndex(1)
                && let Some(url) = item.fileURLValue()
                && let Some(path) = url.path()
            {
                DOCUMENT_DROPS.with(|slot| {
                    if let Some(drops) = slot.borrow().as_ref()
                        && let Some(open) = drops.on_drop.borrow().as_ref()
                    {
                        open(PathBuf::from(path.to_string()));
                    }
                });
            }
        }
    }
);

impl OpenDocuments {
    fn register_events(&self) {
        // AppKit installs its document handlers while launching. Register
        // again at the launch notification so its default cannot replace us.
        unsafe {
            NSAppleEventManager::sharedAppleEventManager()
                .setEventHandler_andSelector_forEventClass_andEventID(
                    self,
                    sel!(handleDocuments:withReplyEvent:),
                    u32::from_be_bytes(*b"aevt"),
                    u32::from_be_bytes(*b"odoc"),
                );
        }
    }
}

pub fn install_open_documents(drops: Rc<crate::backend::Drops>) {
    DOCUMENT_DROPS.with(|slot| *slot.borrow_mut() = Some(drops));
    let handler: Retained<OpenDocuments> = unsafe { msg_send![OpenDocuments::alloc(), init] };
    // Keep winit's NSApplication delegate. Observe launch notifications
    // rather than replacing it. The retained handler outlives registration.
    unsafe {
        for name in [
            ns_string!("NSApplicationWillFinishLaunchingNotification"),
            ns_string!("NSApplicationDidFinishLaunchingNotification"),
        ] {
            NSNotificationCenter::defaultCenter().addObserver_selector_name_object(
                &handler,
                sel!(applicationLaunching:),
                Some(name),
                None,
            );
        }
    }
    handler.register_events();
    DOCUMENT_HANDLER.with(|slot| *slot.borrow_mut() = Some(handler));
}
