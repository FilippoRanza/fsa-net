mod full_space;
mod linspace;
mod run;


pub use run::run;
pub use full_space::FullSpaceResult;
pub use linspace::LinSpaceResult;


pub enum NetworkResult {
    FullSpace(full_space::FullSpaceResult),
    Linspace(linspace::LinSpaceResult)
}

