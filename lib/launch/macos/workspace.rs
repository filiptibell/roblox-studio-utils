use std::{path::Path, sync::mpsc, time::Duration};

use block2::RcBlock;
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSRunningApplication, NSWorkspace, NSWorkspaceOpenConfiguration};
use objc2_foundation::{NSArray, NSDictionary, NSError, NSString, NSURL};

use crate::launch::Launch;
use crate::result::{RobloxStudioError, RobloxStudioResult};

/**
    Stops Qt from activating the app when it launches, and when it raises a window,
    using `activateIgnoringOtherApps:` - this would steal focus, and unhide the app.
*/
const QT_ENVIRONMENT: [(&str, &str); 2] = [
    ("QT_MAC_DISABLE_FOREGROUND_APPLICATION_TRANSFORM", "1"),
    ("QT_MAC_SET_RAISE_PROCESS", "0"),
];

const LAUNCH_TIMEOUT: Duration = Duration::from_mins(1);

type CompletionResult = RobloxStudioResult<libc::pid_t>;

/**
    Launches a new instance of the app, hidden and without activating it, and returns its pid.

    Only the system can launch an app hidden - it does not let other processes hide
    an app after launch - so the system launches the app, and not the current process.
*/
pub(super) fn open_hidden(launch: &Launch) -> CompletionResult {
    // The executable is at `<bundle>.app/Contents/MacOS/<name>`
    let bundle = launch
        .exe
        .ancestors()
        .nth(3)
        .filter(|bundle| bundle.extension().is_some_and(|ext| ext == "app"))
        .and_then(Path::to_str)
        .ok_or_else(|| launch_error("executable is not inside an app bundle"))?;
    let args = launch
        .args
        .iter()
        .map(|arg| arg.to_str().map(NSString::from_str))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| launch_error("argument is not valid UTF-8"))?;

    let (tx, rx) = mpsc::channel::<CompletionResult>();
    let handler = completion_handler(move |app, error| {
        let result = match (app, error) {
            (Some(app), _) if app.processIdentifier() > 0 => Ok(app.processIdentifier()),
            (_, Some(error)) => Err(launch_error(error.localizedDescription().to_string())),
            _ => Err(launch_error("launch finished without a process")),
        };
        // Nothing can stop an app that launches after the timeout, so it must not keep running
        if let Err(mpsc::SendError(Ok(pid))) = tx.send(result) {
            super::signal(pid, libc::SIGKILL).ok();
        }
    });

    autoreleasepool(|_| {
        let config = NSWorkspaceOpenConfiguration::configuration();
        config.setActivates(false);
        config.setHides(true);
        config.setCreatesNewApplicationInstance(true);
        config.setAddsToRecentItems(false);
        config.setPromptsUserIfNeeded(false);
        config.setArguments(&NSArray::from_retained_slice(&args));

        let keys = QT_ENVIRONMENT.map(|(key, _)| NSString::from_str(key));
        let values = QT_ENVIRONMENT.map(|(_, value)| NSString::from_str(value));
        let keys = keys.each_ref().map(|key| &**key);
        config.setEnvironment(&NSDictionary::from_retained_objects(&keys, &values));

        let url = NSURL::fileURLWithPath(&NSString::from_str(bundle));
        NSWorkspace::sharedWorkspace().openApplicationAtURL_configuration_completionHandler(
            &url,
            &config,
            Some(&handler),
        );
    });

    rx.recv_timeout(LAUNCH_TIMEOUT)
        .map_err(|_| launch_error("timed out waiting for the launch to finish"))?
}

/**
    Creates the completion handler block for `openApplicationAtURL:configuration:completionHandler:`.

    The system calls the block on a thread of its own, so the closure must be `Send + Sync`.
*/
fn completion_handler<F>(f: F) -> RcBlock<dyn Fn(*mut NSRunningApplication, *mut NSError)>
where
    F: Fn(Option<&NSRunningApplication>, Option<&NSError>) + Send + Sync + 'static,
{
    RcBlock::new(move |app: *mut NSRunningApplication, error: *mut NSError| {
        // SAFETY: The system passes null, or a valid object that lives for the whole call
        let app = unsafe { app.as_ref() };
        // SAFETY: The system passes null, or a valid object that lives for the whole call
        let error = unsafe { error.as_ref() };
        f(app, error);
    })
}

fn launch_error(message: impl Into<String>) -> RobloxStudioError {
    RobloxStudioError::BackgroundLaunch(message.into())
}
