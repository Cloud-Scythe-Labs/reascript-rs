use std::{ffi::CString, path::PathBuf};

use reaper_medium::{
    BorrowedPcmSource, EnumProjectsResult, MediaItemTake, ProjectContext, ProjectRef, ReaProject,
    Reaper,
};

/// The size of the buffers REAPER writes file paths into, NUL included. A longer path doesn't
/// fit. 4096 bytes is Linux's `PATH_MAX`, four times macOS's, and far more than Windows' classic
/// `MAX_PATH` of 260 characters.
pub const MAX_FILE_PATH_BUFFER_LEN: u32 = 4096;

/// The REAPER session file
#[derive(Debug, Clone)]
pub struct RppFile(PathBuf);
impl RppFile {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Get the name of the file without the extension
    pub fn name(&self) -> Option<String> {
        self.0.file_name().and_then(|n| {
            n.to_string_lossy()
                .rsplit_once('.')
                .map(|(n, _)| n.to_string())
        })
    }

    /// Path to the session file (`.RPP`)
    pub fn path(&self) -> &PathBuf {
        &self.0
    }

    /// Get the last modification time in UTC
    pub fn modtime(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        get_modtime(&self.0)
    }
}

/// Top level project info for attribute selection
#[derive(Debug, Clone)]
pub struct Project {
    /// The name of the project file, without the `.RPP` file ext
    name: String,
    /// A raw pointer to the project, use with caution
    raw: reaper_medium::ReaProject,
    /// The path to the REAPER Media folder
    media_path: PathBuf,
    /// The REAPER session file
    file: RppFile,
    /// List of project [`Track`] information
    tracks: Vec<Track>,
}
impl Project {
    /// Load information for the top level project for attribute selection
    pub fn load(reaper: &Reaper) -> Option<Self> {
        if let Some(current_project) = get_current_project(reaper)
            && let Some(rpp_filepath) = get_rpp_file_path(&current_project)
        {
            let context = ProjectContext::Proj(current_project.project);
            let file = RppFile::new(rpp_filepath);

            return Some(Self {
                name: file.name()?,
                raw: current_project.project,
                media_path: reaper
                    .get_project_path_ex(context, MAX_FILE_PATH_BUFFER_LEN)
                    .into_std_path_buf(),
                file,
                tracks: get_tracks_for_project(reaper, context),
            });
        }
        None
    }

    /// The file name of the project
    pub fn name(&self) -> &str {
        &self.name
    }

    /// A wrapper around the raw project pointer, use with caution
    pub fn raw(&self) -> &reaper_medium::ReaProject {
        &self.raw
    }

    /// The path to the REAPER Media folder
    pub fn media_path(&self) -> &PathBuf {
        &self.media_path
    }

    /// The REAPER session file
    pub fn file(&self) -> &RppFile {
        &self.file
    }

    /// Return a slice of [`Track`]s
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// True if the project doesn't contain any tracks
    pub fn is_empty(&self) -> bool {
        self.tracks().len() > 0
    }

    /// Check if the project scope has changed.
    pub fn is_current_project(&self, current_project: &Project) -> bool {
        self.name() == current_project.name()
    }
}

/// A music track in a REAPER project, containing some sort of media source files
#[derive(Debug, Clone)]
pub struct Track {
    name: String,
    items: Vec<Item>,
}
impl Track {
    pub fn new(name: String, items: Vec<Item>) -> Self {
        Self { name, items }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

#[derive(Default)]
pub struct TrackBuilder {
    name: Option<String>,
    items: Vec<Item>,
}
impl TrackBuilder {
    pub fn name(self, name: Option<String>) -> Self {
        let mut buf = self;
        buf.name = name;
        buf
    }
    pub fn items(self, items: Vec<Item>) -> Self {
        let mut buf = self;
        buf.items = items;
        buf
    }
    pub fn build(self) -> Option<Track> {
        self.name.map(|name| Track::new(name, self.items))
    }
}

/// The media beloning to some [`Track`]
#[derive(Debug, Clone)]
pub struct Item {
    active_take: Take,
    takes: Vec<Take>,
}
impl Item {
    pub fn new(active_take: Take, takes: Vec<Take>) -> Self {
        Self { active_take, takes }
    }
    /// The currently active [`Take`] is a Take which is heard when audio is played back.
    /// This is useful for getting the name of the [`Item`].
    pub fn active_take(&self) -> &Take {
        &self.active_take
    }

    /// A list of all the [`Take`]s in an [`Item`].
    pub fn takes(&self) -> &[Take] {
        &self.takes
    }
}

#[derive(Default)]
pub struct ItemBuilder {
    active_take: Option<Take>,
    takes: Vec<Take>,
}
impl ItemBuilder {
    pub fn active_take(self, active_take: Option<Take>) -> Self {
        let mut buf = self;
        buf.active_take = active_take;
        buf
    }
    pub fn takes(self, takes: Vec<Take>) -> Self {
        let mut buf = self;
        buf.takes = takes;
        buf
    }
    pub fn build(self) -> Option<Item> {
        self.active_take
            .map(|active_take| Item::new(active_take, self.takes))
    }
}

/// Some iteration of a form of recorded media.
/// For instance, "take 1" is the first recording of some media.
#[derive(Debug, Clone)]
pub struct Take {
    name: String,
    source: Option<Source>,
}
impl Take {
    pub fn new(name: String, source: Option<Source>) -> Self {
        Self { name, source }
    }
    /// The name of the [`Take`]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The source file that is played back in the REAPER session
    pub fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }
}
#[derive(Default)]
pub struct TakeBuilder {
    name: Option<String>,
    // TODO: MIDI sources don't exist, they are embedded within REAPER's session data.
    // We should try to match on these kinds of things and export the sources, otherwise
    // it will be much harder for users to revert if someone changes MIDI, they would have
    // to revert the session itself which could be a lot of work.
    source: Option<Source>,
}
impl TakeBuilder {
    pub fn name(self, name: Option<String>) -> Self {
        let mut buf = self;
        buf.name = name;
        buf
    }
    pub fn source(self, source: Option<Source>) -> Self {
        let mut buf = self;
        buf.source = source;
        buf
    }
    pub fn build(self) -> Option<Take> {
        self.name.map(|name| Take::new(name, self.source))
    }
}

/// The file that is played back in the REAPER session
#[derive(Debug, Clone)]
pub struct Source {
    file_path: Option<PathBuf>,
    r#type: String,
}
impl Source {
    pub fn new(file_path: Option<PathBuf>, source_type: String) -> Self {
        Self {
            r#type: source_type,
            file_path,
        }
    }

    /// The file path of the source file for some [`Take`]
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// The file name of the source file for some [`Take`]
    pub fn file_name(&self) -> Option<&std::ffi::OsStr> {
        self.file_path
            .as_ref()
            .and_then(|file_path| file_path.file_name())
    }

    /// The type of the source file, eg. MIDI, WAV, etc.
    pub fn file_type(&self) -> &str {
        &self.r#type
    }
}
#[derive(Default)]
pub struct SourceBuilder {
    file_path: Option<PathBuf>,
    r#type: Option<String>,
}
impl SourceBuilder {
    pub fn file_path(self, file_path: Option<PathBuf>) -> Self {
        let mut buf = self;
        buf.file_path = file_path;
        buf
    }
    pub fn source_type(self, source_type: Option<String>) -> Self {
        let mut buf = self;
        buf.r#type = source_type;
        buf
    }
    pub fn build(self) -> Option<Source> {
        self.r#type
            .map(|source_type| Source::new(self.file_path, source_type))
    }
}

/// Get the currently active project and its filepath.
pub fn get_current_project(reaper: &Reaper) -> Option<EnumProjectsResult> {
    reaper.enum_projects(ProjectRef::Current, MAX_FILE_PATH_BUFFER_LEN)
}

/// Get all of the [`Track`]s from a session. In general it's best to call `Project::load`
/// to get the list of [`Track`]s, but this could be used if the extra information from [`Project`]
/// isn't needed.
///
/// Use [`get_current_project`] to get access to the `reaper_medium::ProjectContext` for the current project.
pub fn get_tracks_for_project(reaper: &Reaper, context: ProjectContext) -> Vec<Track> {
    // TODO: The get_track method returns an option so we filter_map here for convenience,
    // however, this should never return None since the index count for the track should be
    // accurate. If it ever does return None we should be handling this edge case and have
    // users report it, since it would cause the plugin to misbehave.
    (0..reaper.count_tracks(context))
        .filter_map(|track_index| {
            reaper
                .get_track(context, track_index)
                .and_then(|media_track| {
                    TrackBuilder::default()
                        .name(
                            unsafe {
                                reaper.get_set_media_track_info_get_name(media_track, |name| {
                                    name.to_owned()
                                })
                            }
                            .map(|name| {
                                let name = name.to_string();
                                (!name.is_empty())
                                    .then_some(name)
                                    .unwrap_or(format!("Track-{}", track_index + 1))
                            }),
                        )
                        .items(get_track_items(reaper, media_track))
                        .build()
                })
        })
        .collect()
}

pub fn get_track_items(reaper: &Reaper, media_track: reaper_medium::MediaTrack) -> Vec<Item> {
    (0..unsafe { reaper.count_track_media_items(media_track) })
        .filter_map(|item_index| {
            unsafe { reaper.get_track_media_item(media_track, item_index) }.and_then(|media_item| {
                unsafe { reaper.get_active_take(media_item) }.and_then(|active_take| {
                    ItemBuilder::default()
                        .active_take(
                            TakeBuilder::default()
                                .name(get_take_name(reaper, active_take))
                                .source(
                                    get_media_item_take_source_path_and_type(reaper, active_take)
                                        .and_then(|(file_path, source_type)| {
                                            SourceBuilder::default()
                                                .file_path(file_path)
                                                .source_type(Some(source_type))
                                                .build()
                                        }),
                                )
                                .build(),
                        )
                        .takes(get_media_item_takes(reaper, media_item))
                        .build()
                })
            })
        })
        .collect()
}

/// Get the `reaper_medium::MediaItemTake` name as a `std::str::String` if it exists.
pub fn get_take_name(
    reaper: &Reaper,
    media_item_take: reaper_medium::MediaItemTake,
) -> Option<String> {
    reaper
        .get_take_name(media_item_take, |name| name.map(|name| name.to_owned()))
        .map(|name| name.to_string())
        .ok()
}

/// Get the `reaper_medium::MediaItemTake` source path as a `std::path::PathBuf`.
pub fn get_media_item_take_source_path_and_type(
    reaper: &Reaper,
    media_item_take: reaper_medium::MediaItemTake,
) -> Option<(Option<PathBuf>, String)> {
    unsafe { reaper.get_media_item_take_source(media_item_take) }.map(|pcm_source| {
        let raw_source_ptr = pcm_source.as_ptr();
        (
            unsafe { raw_source_ptr.as_ref() }.and_then(|raw_source_ptr| {
                BorrowedPcmSource::from_raw(raw_source_ptr)
                    .get_file_name(|file| file.map(|file| file.to_path_buf().into_std_path_buf()))
            }),
            util::with_string_buffer(MAX_FILE_PATH_BUFFER_LEN, |buffer, max_size| unsafe {
                reaper
                    .low()
                    .GetMediaSourceType(raw_source_ptr, buffer, max_size)
            })
            .0
            .into_string(),
        )
    })
}

/// Get all [`Take`]s for a `reaper_medium::MediaItem`.
pub fn get_media_item_takes(reaper: &Reaper, media_item: reaper_medium::MediaItem) -> Vec<Take> {
    (0..(unsafe { reaper.low().GetMediaItemNumTakes(media_item.as_ptr()) } as u32))
        .filter_map(|take_index| {
            MediaItemTake::new(unsafe {
                reaper.low().GetTake(media_item.as_ptr(), take_index as i32)
            })
            .and_then(|media_item_take| {
                TakeBuilder::default()
                    .name(get_take_name(reaper, media_item_take))
                    .source(
                        get_media_item_take_source_path_and_type(reaper, media_item_take).and_then(
                            |(file_path, source_type)| {
                                SourceBuilder::default()
                                    .file_path(file_path)
                                    .source_type(Some(source_type))
                                    .build()
                            },
                        ),
                    )
                    .build()
            })
        })
        .collect()
}

/// Get the path to the `.RPP` file from a project.
pub fn get_rpp_file_path(project: &EnumProjectsResult) -> Option<PathBuf> {
    project
        .file_path
        .as_ref()
        .filter(|file_path| !file_path.as_str().is_empty())
        .map(|file_path| file_path.into())
}

// TODO: Need to test this to ensure it's working properly across all supported systems
/// Get the last modification timestamp for a file.
pub fn get_modtime(filepath: &PathBuf) -> Option<chrono::DateTime<chrono::Utc>> {
    let meta = std::fs::metadata(filepath).ok()?;
    let timestamp = filetime::FileTime::from_last_modification_time(&meta);
    chrono::DateTime::from_timestamp(timestamp.unix_seconds(), timestamp.nanoseconds())
}

/// Get the value associated with this extname and key in the project's extension state: the last
/// one set with [`set_proj_ext_state`], or the one loaded from the session file. `None` if there
/// is none, it's empty or it isn't UTF-8. Only values shorter than `buffer_size` bytes are read
/// whole.
pub fn get_proj_ext_state(
    reaper: &Reaper,
    project: ReaProject,
    extname: &str,
    key: &str,
    buffer_size: u32,
) -> Option<String> {
    let extname = CString::new(extname).ok()?;
    let key = CString::new(key).ok()?;
    let (value, _) = util::with_string_buffer_cstring(buffer_size, |buffer, max_size| unsafe {
        reaper.low().GetProjExtState(
            project.as_ptr(),
            extname.as_ptr(),
            key.as_ptr(),
            buffer,
            max_size,
        )
    });
    value.into_string().ok().filter(|value| !value.is_empty())
}

/// Save a key/value pair for a specific extension, to be restored the next time this specific project is loaded.
/// Typically extname will be the name of a reascript or extension section. If key is NULL or "", all extended
/// data for that extname will be deleted. If val is NULL or "", the data previously associated with that key
/// will be deleted. Returns the size of the state for this extname, or `None` if a string contains a NUL byte.
pub fn set_proj_ext_state(
    reaper: &Reaper,
    project: ReaProject,
    extname: &str,
    key: &str,
    value: &str,
) -> Option<i32> {
    let extname = CString::new(extname).ok()?;
    let key = CString::new(key).ok()?;
    let value = CString::new(value).ok()?;

    Some(unsafe {
        reaper.low().SetProjExtState(
            project.as_ptr(),
            extname.as_ptr(),
            key.as_ptr(),
            value.as_ptr(),
        )
    })
}

/// Returns true if there exists an extended state value for a specific section and key, i.e. in
/// REAPER's global extension state. False if a string contains a NUL byte.
pub fn has_ext_state(reaper: &Reaper, section: &str, key: &str) -> bool {
    let (Ok(section), Ok(key)) = (CString::new(section), CString::new(key)) else {
        return false;
    };
    unsafe { reaper.low().HasExtState(section.as_ptr(), key.as_ptr()) }
}

/// Set the extended state value for a specific section and key, i.e. in REAPER's global extension
/// state. `persist` means the value should be stored (in `reaper-extstate.ini`) and reloaded the
/// next time REAPER is opened. Does nothing if a string contains a NUL byte.
pub fn set_ext_state(reaper: &Reaper, section: &str, key: &str, value: &str, persist: bool) {
    let (Ok(section), Ok(key), Ok(value)) = (
        CString::new(section),
        CString::new(key),
        CString::new(value),
    ) else {
        return;
    };
    unsafe {
        reaper
            .low()
            .SetExtState(section.as_ptr(), key.as_ptr(), value.as_ptr(), persist);
    }
}

/// Delete the extended state value for a specific section and key, i.e. from REAPER's global
/// extension state. `persist` means the value should remain deleted the next time REAPER is
/// opened. Does nothing if a string contains a NUL byte.
pub fn delete_ext_state(reaper: &Reaper, section: &str, key: &str, persist: bool) {
    let (Ok(section), Ok(key)) = (CString::new(section), CString::new(key)) else {
        return;
    };
    unsafe {
        reaper
            .low()
            .DeleteExtState(section.as_ptr(), key.as_ptr(), persist);
    }
}

/// Save the project, like REAPER's "Save project" action: a project that was never saved, or any
/// project if `force_save_as`, asks the user where to save it.
pub fn save_project(reaper: &Reaper, project: ReaProject, force_save_as: bool) {
    unsafe {
        reaper
            .low()
            .Main_SaveProject(project.as_ptr(), force_save_as)
    }
}

mod util {
    use reaper_medium::ReaperString;
    use std::ffi::CString;
    use std::os::raw::c_char;

    pub fn with_string_buffer<T>(
        max_size: u32,
        fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
    ) -> (ReaperString, T) {
        let (cstring, result) = with_string_buffer_cstring(max_size, fill_buffer);
        (unsafe { ReaperString::new_unchecked(cstring) }, result)
    }

    pub fn with_string_buffer_cstring<T>(
        max_size: u32,
        fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
    ) -> (CString, T) {
        // Using with_capacity() here wouldn't be correct because it leaves the vector length at zero.
        let vec: Vec<u8> = vec![0; max_size as usize];
        with_string_buffer_internal(vec, max_size, fill_buffer)
    }

    fn with_string_buffer_internal<T>(
        vec: Vec<u8>,
        max_size: u32,
        fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
    ) -> (CString, T) {
        let c_string = unsafe { CString::from_vec_unchecked(vec) };
        let raw = c_string.into_raw();
        let result = fill_buffer(raw, max_size as i32);
        let cstring = unsafe { CString::from_raw(raw) };
        (cstring, result)
    }
}
