use std::io::{Read, Result, BufReader, Cursor, Seek};
use std::path::{Path, PathBuf};
use std::fs::{self, File, FileType};
use zip::ZipArchive;
use zip::read::ZipFile;
use crate::util::InnerMatches;

pub struct Sound<Index: Clone + Send> {
    pub rating: f32,
    pub index: Index,
}

impl<Index: Clone + Send + std::fmt::Debug> std::fmt::Debug for Sound<Index> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sound")
            .field("rating", &format!("{:.2}%", &self.rating * 100f32))
            .field("index", &self.index)
            .finish()
    }
}

pub trait Loader {
    type Index: Clone + Send;
    type Output: Read + Send;

    fn all(self) -> Result<Vec<Sound<Self::Index>>>;
    fn load(self, index: &Self::Index) -> Result<Self::Output>;
}

pub struct DirectoryLoader<P: AsRef<Path>>(P);

impl<P: AsRef<Path>> DirectoryLoader<P> {
    pub fn new(path: P) -> Self {
        Self(path)
    }
}

impl<P: AsRef<Path>> Loader for &DirectoryLoader<P> {
    type Index = PathBuf;
    type Output = BufReader<File>;

    fn all(self) -> Result<Vec<Sound<Self::Index>>> {
        let dir = fs::read_dir(&self.0)?;

        Ok(dir.filter_map(|entry| entry.ok()).filter(|entry| {
            entry.file_type().inner_is(|ft| ft.is_file())
            && entry.path().extension().inner_is(|ext| ext == "wav")
        }).map(|file| Sound {
            rating: Default::default(),
            index: file.path(),
        }).collect())
    }

    fn load(self, index: &Self::Index) -> Result<Self::Output> {
        Ok(BufReader::new(File::open(index)?))
    }
}

pub struct ZipLoader<R: Read + Seek>(ZipArchive<R>);

impl<R: Read + Seek> ZipLoader<R> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self(zip::ZipArchive::new(reader).map_err::<std::io::Error, _>(Into::into)?))
    }
}

impl<R: Read + Seek> Loader for &mut ZipLoader<R> {
    type Index = usize;
    type Output = Cursor<Vec<u8>>;

    fn all(self) -> Result<Vec<Sound<Self::Index>>> {
        Ok(self.0.file_names().enumerate().filter_map(|(index, name)| {
            if name.ends_with(".wav") {
                Some(Sound {
                    rating: Default::default(),
                    index,
                })
            } else { None }
        }).collect())
    }

    fn load(self, index: &Self::Index) -> Result<Self::Output> {
        self.0.by_index(*index).map_err(Into::into).map(|mut file| {
            let mut vec = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut vec);
            Cursor::new(vec)
        })
    }
}
