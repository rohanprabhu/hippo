extern crate colored;

use self::colored::*;

lazy_static! {
    pub static ref HIPPO_PRINTABLE : ColoredString = {
        "hippo".magenta()
    };
}
