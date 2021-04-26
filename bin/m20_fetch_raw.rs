
use mars_raw_utils::{
    constants, 
    print, 
    util,
    m20remote
};

#[macro_use]
extern crate clap;
use std::process;
use clap::{Arg, App};

fn main() {
    let matches = App::new(crate_name!())
                    .version(crate_version!())
                    .author(crate_authors!())
                .arg(Arg::with_name(constants::param::PARAM_VERBOSE)
                    .short(constants::param::PARAM_VERBOSE)
                    .help("Show verbose output"))
                .arg(Arg::with_name("camera")
                    .short("c")
                    .long("camera")
                    .value_name("camera")
                    .help("M20 Camera Instrument(s)")
                    .required(false)
                    .takes_value(true)
                    .multiple(true))
                .arg(Arg::with_name("sol")
                    .short("s")
                    .long("sol")
                    .value_name("sol")
                    .help("Mission Sol")
                    .required(false)
                    .takes_value(true))    
                .arg(Arg::with_name("minsol")
                    .short("m")
                    .long("minsol")
                    .value_name("minsol")
                    .help("Starting Mission Sol")
                    .required(false)
                    .takes_value(true))  
                .arg(Arg::with_name("maxsol")
                    .short("M")
                    .long("maxsol")
                    .value_name("maxsol")
                    .help("Ending Mission Sol")
                    .required(false)
                    .takes_value(true)) 
                .arg(Arg::with_name("list")
                    .short("l")
                    .long("list")
                    .value_name("list")
                    .help("Don't download, only list results")
                    .takes_value(false)
                    .required(false)) 
                .arg(Arg::with_name("movie")
                    .short("e")
                    .long("movie")
                    .value_name("movie")
                    .help("Only movie frames")
                    .takes_value(false)
                    .required(false)) 
                .arg(Arg::with_name("thumbnails")
                    .short("t")
                    .long("thumbnails")
                    .value_name("thumbnails")
                    .help("Download thumbnails in the results")
                    .takes_value(false)
                    .required(false)) 
                .arg(Arg::with_name("num")
                    .short("N")
                    .long("num")
                    .value_name("num")
                    .help("Max number of results")
                    .required(false)
                    .takes_value(true))    
                .arg(Arg::with_name("page")
                    .short("p")
                    .long("page")
                    .value_name("page")
                    .help("Results page (starts at 1)")
                    .required(false)
                    .takes_value(true))  
                .arg(Arg::with_name("seqid")
                    .short("S")
                    .long("seqid")
                    .value_name("seqid")
                    .help("Specific sequence id or substring")
                    .required(false)
                    .takes_value(true))  
                .arg(Arg::with_name("instruments")
                    .short("i")
                    .long("instruments")
                    .value_name("instruments")
                    .help("List camera instrument and exit")
                    .takes_value(false)
                    .required(false)) 
                .arg(Arg::with_name(constants::param::PARAM_ONLY_NEW)
                    .short(constants::param::PARAM_ONLY_NEW_SHORT)
                    .help("Only new images. Skipped processed images."))
                .get_matches();


    let im = m20remote::make_instrument_map();

    if matches.is_present(constants::param::PARAM_VERBOSE) {
        print::set_verbose(true);
    }

    if matches.is_present("instruments") {
        util::print_instruments(&im.map);
        process::exit(0);
    }

    let mut num_per_page = 100;
    let mut page = None;
    let mut minsol = 1000000;
    let mut maxsol = -1;
    let mut sol = -1;
    let mut thumbnails = false;
    let mut search = "";
    let mut list_only = false;
    let mut movie_only = false;

    let only_new = matches.is_present(constants::param::PARAM_ONLY_NEW);

    let mut camera_inputs: Vec<&str> = Vec::default();
    if matches.is_present("camera") {
        camera_inputs = matches.values_of("camera").unwrap().collect();
    }

    let camera_ids_res = util::find_remote_instrument_names_fromlist(&camera_inputs, &im.map);
    let cameras = match camera_ids_res {
        Err(_e) => {
            eprintln!("Invalid camera instrument(s) specified");
            process::exit(1);
        },
        Ok(v) => v,
    };
    
    if matches.is_present("thumbnails") {
        thumbnails = true;
    }

    if matches.is_present("movie") {
        movie_only = true;
    }

    if matches.is_present("list") {
        list_only = true;
    }

    if matches.is_present("seqid") {
        search =  matches.value_of("seqid").unwrap();
    }

    if matches.is_present("num") {
        let s = matches.value_of("num").unwrap();
        if util::string_is_valid_i32(&s) {
            num_per_page = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if matches.is_present("page") {
        let s = matches.value_of("page").unwrap();
        if util::string_is_valid_i32(&s) {
            page = Some(s.parse::<i32>().unwrap() - 1);
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if matches.is_present("minsol") {
        let s = matches.value_of("minsol").unwrap();
        if util::string_is_valid_i32(&s) {
            minsol = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if matches.is_present("maxsol") {
        let s = matches.value_of("maxsol").unwrap();
        if util::string_is_valid_i32(&s) {
            maxsol = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if matches.is_present("sol") {
        let s = matches.value_of("sol").unwrap();
        if util::string_is_valid_i32(&s) {
            sol = s.parse::<i32>().unwrap();
        } else {
            eprintln!("Error: Invalid number specified");
            process::exit(1);
        }
    }

    if sol >= 0 {
        minsol = sol;
        maxsol = sol;
    }

    m20remote::print_header();
    match m20remote::remote_fetch(&cameras, num_per_page, page, minsol, maxsol, thumbnails, movie_only, list_only, search, only_new) {
        Ok(c) => println!("{} images found", c),
        Err(e) => eprintln!("Error: {}", e)
    };
}
