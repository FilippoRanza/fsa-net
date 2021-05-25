mod full_space;
mod run;


pub use run::run;
pub use full_space::FullSpaceResult;


pub enum NetworkResult {
    FullSpace(full_space::FullSpaceResult)
}

