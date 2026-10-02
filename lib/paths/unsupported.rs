use crate::{RobloxStudioError, RobloxStudioResult};

use super::RobloxStudioPathsInner;

impl RobloxStudioPathsInner {
    pub(super) fn new() -> RobloxStudioResult<Self> {
        Err(RobloxStudioError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use crate::{RobloxStudioError, RobloxStudioPaths};

    #[test]
    fn new_returns_unsupported_platform() {
        assert!(matches!(
            RobloxStudioPaths::new(),
            Err(RobloxStudioError::UnsupportedPlatform)
        ));
    }
}
