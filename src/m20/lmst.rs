use crate::{
    time,
    constants,
    error
};

pub fn get_lmst() -> error::Result<time::MissionTime> {
    time::get_lmst(constants::time::M2020_SOL_OFFSET, constants::time::M2020_LONGITUDE)
}