use crate::error::Error;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::Path;
use tempfile::tempdir_in;

/// Supported archive types.
pub(crate) enum ArchiveFormat {
    TarGz,
    Zip,
}

impl ArchiveFormat {
    /// Parse archive type from resource extension.
    pub(crate) fn parse_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let ext = infer::get_from_path(path).unwrap().unwrap().extension();
        // Here we assume that gz contain a tar inside it.
        if ext == "gz" {
            Ok(Self::TarGz)
        } else if ext == "zip" {
            Ok(Self::Zip)
        } else {
            Err(Error::ExtractionError("unsupported archive format".into()))
        }
    }
}

pub(crate) fn extract_archive<P: AsRef<Path>>(
    path: P,
    target: P,
    format: &ArchiveFormat,
) -> Result<(), Error> {
    // We'll first extract to a temp directory in the same parent as the target directory.
    let target_parent_dir = target.as_ref().parent().unwrap();
    let temp_target = tempdir_in(target_parent_dir)?;

    match format {
        ArchiveFormat::TarGz => {
            let tar_gz = File::open(&path)?;
            let tar = GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(tar);
            archive.unpack(&temp_target)?;
        }
        ArchiveFormat::Zip => {
            let file = File::open(&path)?;
            let mut archive =
                zip::ZipArchive::new(file).map_err(|e| Error::ExtractionError(e.to_string()))?;
            archive
                .extract(temp_target.path())
                .map_err(|e| Error::ExtractionError(e.to_string()))?;
        }
    };

    fs::remove_file(&path)?;
    // Now rename the temp directory to the final target directory.
    fs::rename(temp_target, target)?;

    Ok(())
}
