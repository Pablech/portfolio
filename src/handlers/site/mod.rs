mod home;
mod about;
mod help;
mod keys;
mod resume;
mod email;
mod github;
mod game;
mod links;

pub use home::home;
pub use about::about;
pub use help::help;
pub use keys::keys;
pub use resume::resume;
pub use email::send_email;
pub use github::github;
pub use game::*;
pub use links::links;