use crate::constants;

use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum TimeSystem {
    LMST,
    HLST,
}

fn within_24(n: f64) -> f64 {
    let mut _n = n;
    if _n < 0.0 {
        _n += 24.0;
    } else if _n >= 24.0 {
        _n -= 24.0;
    }
    _n
}

pub struct Hms {
    pub hours: f64,
    pub minutes: f64,
    pub seconds: f64,
}

pub struct MissionTime {
    pub mission_time_display: String,
    pub ltst_display: String,
    pub mtc_display: String,
    pub sol: i32,
    pub mission_time_hms: Hms,
    pub ltst_hms: Hms,
    pub sclk: i32,
    pub msd: f64,
    pub mtc: f64,
    pub mtc_hms: Hms,
    pub mission_time: f64,
    pub ltst: f64,
    pub l_s: f64,
    pub nu: f64,
    pub e: f64,
    pub time_system: TimeSystem,
}

pub fn get_seconds_since_epoch() -> f64 {
    let now = SystemTime::now();

    let unix_time = now.duration_since(UNIX_EPOCH).unwrap();
    let unix_secs = unix_time.as_secs() as f64;
    let unix_millis = ((unix_time.as_nanos() % 1_000_000_000) as f64) / 1_000_000_000.0;
    unix_secs + unix_millis
}

// NOTE: This isn't accurate.
pub fn get_lmst_from_epoch_secs(epoch: f64, longitude: f64) -> Result<MissionTime> {
    let jd_land = 2440587.5 + (epoch * 1000.0 / 8.64E7);
    let jd_tt_land = jd_land + (constants::time::TAI_OFFSET) / 86400.0;
    let j2000_land = jd_tt_land - 2451545.0 + 0.00014;
    let sol_offset = ((j2000_land - 4.5) / 1.027491252) + 44796.0 - 0.00096;

    get_time(-1.0 * sol_offset, longitude, TimeSystem::LMST)
}

fn cos(v: f64) -> f64 {
    (v * std::f64::consts::PI / 180.0).cos()
}

fn sin(v: f64) -> f64 {
    (v * std::f64::consts::PI / 180.0).sin()
}

fn t_to_hms(t: f64) -> Hms {
    let hours = t.floor();
    let minutes_f = 60.0 * (t - hours);
    let minutes = minutes_f.floor();
    let seconds = 60.0 * (minutes_f - minutes);

    Hms {
        hours,
        minutes,
        seconds,
    }
}

// Based on m2020-bitbar which in turn is based on James Tauber's Mars Clock
// See http://marsclock.com/
pub fn get_time(sol_offset: f64, longitude: f64, time_system: TimeSystem) -> Result<MissionTime> {
    let seconds_since_epoch = get_seconds_since_epoch();
    let millis = seconds_since_epoch * 1000.0;

    let jd_ut = 2440587.5 + (millis / 8.64E7);
    let jd_tt = jd_ut + (constants::time::TAI_OFFSET + 32.184) / 86400.0;
    let j2000 = jd_tt - 2451545.0;

    let m = (19.3870 + 0.52402075 * j2000) % 360.0;

    let alpha_fms = (270.3863 + 0.52403840 * j2000) % 360.0;
    let e = 0.09340 + 2.477E-9 * j2000;

    let pbs = 0.0071 * cos((0.985626 * j2000 / 2.2353) + 49.409)
        + 0.0057 * cos((0.985626 * j2000 / 2.7543) + 168.173)
        + 0.0039 * cos((0.985626 * j2000 / 1.1177) + 191.837)
        + 0.0037 * cos((0.985626 * j2000 / 15.7866) + 21.736)
        + 0.0021 * cos((0.985626 * j2000 / 2.1354) + 15.704)
        + 0.0020 * cos((0.985626 * j2000 / 2.4694) + 95.528)
        + 0.0018 * cos((0.985626 * j2000 / 32.8493) + 49.095);
    let nu_m = (10.691 + 3.0E-7 * j2000) * sin(m)
        + 0.623 * sin(2.0 * m)
        + 0.050 * sin(3.0 * m)
        + 0.005 * sin(4.0 * m)
        + 0.0005 * sin(5.0 * m)
        + pbs;
    let nu = nu_m + m;
    let l_s = (alpha_fms + nu_m) % 360.0;
    let eot = 2.861 * sin(2.0 * l_s) - 0.071 * sin(4.0 * l_s) + 0.002 * sin(6.0 * l_s) - nu_m;

    let msd = ((j2000 - 4.5) / constants::time::MARS_SEC_ADJUSTMENT) + 44796.0 - 0.00096;
    let mtc = (24.0 * msd) % 24.0;

    let sol = match time_system {
        TimeSystem::LMST => ((msd - (360.0 - longitude) / 360.0) + sol_offset).floor(),
        TimeSystem::HLST => msd + sol_offset,
    };
    let mission_time = match time_system {
        TimeSystem::LMST => within_24(mtc - (360.0 - longitude) * 24.0 / 360.0),
        TimeSystem::HLST => (24.0 * sol) % 24.0,
    };

    let ltst = within_24(mission_time + eot * 24.0 / 360.0);

    let mission_time_hms = t_to_hms(mission_time);
    let ltst_hms = t_to_hms(ltst);
    let mtc_hms = t_to_hms(mtc);

    // VALIDATE THIS SECTION. I'M JUST GUESSING
    // let unix_count = seconds_since_epoch - constants::time::M20_UNIX_COUNT_OFFSET;
    // let display_sclk = constants::time::M20_SURFACE_SCLK + unix_count + 2.0;

    let mission_time_string = format!(
        "{:02}:{:02}:{:06.3} LMST",
        mission_time_hms.hours, mission_time_hms.minutes, mission_time_hms.seconds
    );
    let ltst_string = format!(
        "{:02}:{:02}:{:06.3} LTST",
        ltst_hms.hours, ltst_hms.minutes, ltst_hms.seconds
    );
    let mtc_string = format!(
        "{:02}:{:02}:{:06.3}",
        mtc_hms.hours, mtc_hms.minutes, mtc_hms.seconds
    );

    Ok(MissionTime {
        mission_time_display: mission_time_string,
        ltst_display: ltst_string,
        mtc_display: mtc_string,
        sol: sol as i32,
        mission_time_hms,
        ltst_hms,
        sclk: 0_i32,
        msd,
        mtc,
        mtc_hms,
        mission_time,
        ltst,
        l_s,
        nu,
        e,
        time_system,
    })
}
