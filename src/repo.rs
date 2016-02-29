#![allow(dead_code, unused_variables)]
use std::fmt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::{self, Write};
use std::str;

use git2;
use git2::Error as GitError;
use term::color;
use term::color::Color;

use manager::LuigiDir;

/// More Rustacious way of representing a git status
#[derive(Debug,Clone)]
pub enum GitStatus{
    IndexNew, IndexModified , IndexDeleted, IndexRenamed, IndexTypechange,
    WorkingNew, WorkingModified, WorkingDeleted, WorkingTypechange, WorkingRenamed,
    Ignored, Conflict, Current, Unknown
}

impl GitStatus{
    pub fn to_color(&self) -> Color {
        match *self{
        // => write!(f, "{:?}", self)
         GitStatus::Unknown         => color::YELLOW,
         GitStatus::Current         => color::BLUE,
         GitStatus::Conflict        => color::RED,
         GitStatus::WorkingNew      => color::GREEN,
         GitStatus::WorkingModified => color::YELLOW,
         _                          => color::BLUE
        }
    }
}

impl fmt::Display for GitStatus{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        match *self{
        // => write!(f, "{:?}", self)
         GitStatus::Conflict        => write!(f, "X"),
         GitStatus::Current         => write!(f, "+"),
         GitStatus::WorkingNew      => write!(f, "+"),
         GitStatus::WorkingModified => write!(f, "~"),
         GitStatus::Unknown         => write!(f, ""),
         _                          => write!(f, "{:?}", self),

        }
    }
}

impl From<git2::Status> for GitStatus{
    fn from(status:git2::Status) -> Self{
        match status{
            //s if s.contains(git2::STATUS_CURRENT)          => GitStatus::Current,
            s if s.contains(git2::STATUS_INDEX_NEW)        => GitStatus::IndexNew,
            s if s.contains(git2::STATUS_INDEX_MODIFIED)   => GitStatus::IndexModified ,
            s if s.contains(git2::STATUS_INDEX_DELETED)    => GitStatus::IndexDeleted,
            s if s.contains(git2::STATUS_INDEX_RENAMED)    => GitStatus::IndexRenamed,
            s if s.contains(git2::STATUS_INDEX_TYPECHANGE) => GitStatus::IndexTypechange,
            s if s.contains(git2::STATUS_WT_NEW)           => GitStatus::WorkingNew,
            s if s.contains(git2::STATUS_WT_MODIFIED)      => GitStatus::WorkingModified,
            s if s.contains(git2::STATUS_WT_DELETED)       => GitStatus::WorkingDeleted,
            s if s.contains(git2::STATUS_WT_TYPECHANGE)    => GitStatus::WorkingTypechange,
            s if s.contains(git2::STATUS_WT_RENAMED)       => GitStatus::WorkingRenamed,
            s if s.contains(git2::STATUS_IGNORED)          => GitStatus::Ignored,
            s if s.contains(git2::STATUS_CONFLICTED)       => GitStatus::Conflict,
            _                                              => GitStatus::Unknown
        }
    }
}

/// Convenience Wrapper for `git2::Repository`
pub struct Repository{
    /// Git Repository for StorageDir
    pub repo: git2::Repository,
    /// Maps GitStatus to each path
    pub statuses: HashMap<PathBuf, GitStatus>
}

impl Repository {

    pub fn new(path:&Path) -> Result<Self, GitError>{
        let repo = try!(git2::Repository::open(path));
        let statuses = try!(Self::cache_statuses(&repo));
        Ok(
            Repository{
                repo: repo,
                statuses: statuses
            }
          )
    }

    fn cache_statuses(repo:&git2::Repository) -> Result<HashMap<PathBuf, GitStatus>, GitError>{
        use self::GitStatus::*;
        let repo_path = repo.path().parent().unwrap().to_owned();

        let git_statuses = try!(repo.statuses( Some( git2::StatusOptions::new()
                                                     .include_ignored(false)
                                                     .include_untracked(true) )));

        let mut statuses:HashMap<PathBuf,GitStatus> = HashMap::new();

        for entry in git_statuses.iter(){
            let status:GitStatus = entry.status().into();

            if let Some(path) = entry.path(){
                let path = repo_path.join(PathBuf::from(path));
                if path.is_file() {
                    if let Some(parent) = path.parent(){
                        statuses.insert(parent.to_path_buf(), status.to_owned());
                    }
                }
                statuses.insert(path, status);
            }
        }

        Ok(statuses)
    }

    /// Returns the status to a given path
    pub fn get_status(&self,path:&Path) -> GitStatus{
        return self.statuses.get(path).unwrap_or(&GitStatus::Unknown).to_owned()
    }


}
