//! Multimedia hardware keyboard shortcut handler via WM_APPCOMMAND.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaAction {
    PlayPause,
    Play,
    Pause,
    Stop,
    NextTrack,
    PreviousTrack,
    VolumeMute,
    VolumeDown,
    VolumeUp,
}

// Win32 APPCOMMAND constants
pub const APPCOMMAND_VOLUME_MUTE: u16 = 8;
pub const APPCOMMAND_VOLUME_DOWN: u16 = 9;
pub const APPCOMMAND_VOLUME_UP: u16 = 10;
pub const APPCOMMAND_MEDIA_NEXTTRACK: u16 = 11;
pub const APPCOMMAND_MEDIA_PREVIOUSTRACK: u16 = 12;
pub const APPCOMMAND_MEDIA_STOP: u16 = 13;
pub const APPCOMMAND_MEDIA_PLAY_PAUSE: u16 = 14;
pub const APPCOMMAND_MEDIA_PLAY: u16 = 46;
pub const APPCOMMAND_MEDIA_PAUSE: u16 = 47;

const FAPPCOMMAND_MASK: u32 = 0xF000;

/// Extracts the Win32 APPCOMMAND ID from the `LPARAM` of a `WM_APPCOMMAND` message.
pub fn parse_appcommand(lparam: isize) -> Option<MediaAction> {
    let cmd = (((lparam as usize >> 16) as u32) & !FAPPCOMMAND_MASK) as u16;
    match cmd {
        APPCOMMAND_MEDIA_PLAY_PAUSE => Some(MediaAction::PlayPause),
        APPCOMMAND_MEDIA_PLAY => Some(MediaAction::Play),
        APPCOMMAND_MEDIA_PAUSE => Some(MediaAction::Pause),
        APPCOMMAND_MEDIA_STOP => Some(MediaAction::Stop),
        APPCOMMAND_MEDIA_NEXTTRACK => Some(MediaAction::NextTrack),
        APPCOMMAND_MEDIA_PREVIOUSTRACK => Some(MediaAction::PreviousTrack),
        APPCOMMAND_VOLUME_MUTE => Some(MediaAction::VolumeMute),
        APPCOMMAND_VOLUME_DOWN => Some(MediaAction::VolumeDown),
        APPCOMMAND_VOLUME_UP => Some(MediaAction::VolumeUp),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_play_pause() {
        let lparam = (APPCOMMAND_MEDIA_PLAY_PAUSE as usize) << 16;
        assert_eq!(parse_appcommand(lparam as isize), Some(MediaAction::PlayPause));
    }

    #[test]
    fn parses_next_and_previous() {
        let next_lparam = (APPCOMMAND_MEDIA_NEXTTRACK as usize) << 16;
        assert_eq!(parse_appcommand(next_lparam as isize), Some(MediaAction::NextTrack));

        let prev_lparam = (APPCOMMAND_MEDIA_PREVIOUSTRACK as usize) << 16;
        assert_eq!(parse_appcommand(prev_lparam as isize), Some(MediaAction::PreviousTrack));
    }

    #[test]
    fn ignores_unknown_command() {
        let unknown_lparam = (999usize) << 16;
        assert_eq!(parse_appcommand(unknown_lparam as isize), None);
    }
}
