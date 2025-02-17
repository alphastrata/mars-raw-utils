use crate::{constants, httpfetch::HttpFetcher, util::string_is_valid_f64};
use anyhow::Result;
use serde_json::Value;
use string_builder::Builder;

use anyhow::anyhow;

pub struct JsonFetcher {
    fetcher: HttpFetcher,
}

impl JsonFetcher {
    pub fn new(uri: &str) -> Result<JsonFetcher> {
        Ok(JsonFetcher {
            fetcher: match HttpFetcher::new(uri) {
                Ok(it) => it,
                Err(err) => return Err(err),
            },
        })
    }

    pub fn param(&mut self, key: &str, value: &str) {
        _ = self.fetcher.param(key, value);
    }

    pub async fn fetch(&self) -> Result<Value> {
        let json_text = self.fetcher.into_string().await?; //as_string() is also a common name for this.
        Ok(serde_json::from_str(&json_text.text)?)
    }

    pub async fn fetch_str(&self) -> Result<String> {
        Ok(self.fetcher.into_string().await?.text)
    }
}

fn vec_to_str(v: &[f64]) -> String {
    let mut b = Builder::default();

    for item in v {
        b.append(format!("{},", item));
    }

    let mut s = b.string().unwrap();
    if !s.is_empty() {
        s.remove(s.len() - 1);
    }

    format!("({})", s)
}

fn str_to_vec(s: &str) -> Result<Vec<f64>> {
    let mut tuple_vec: Vec<f64> = Vec::new();
    let mut s0 = String::from(s);
    s0.remove(0);
    s0.remove(s0.len() - 1);
    let split = s0.split(',');
    for n in split {
        let n_t = n.trim();
        if string_is_valid_f64(n_t) {
            tuple_vec.push(n_t.parse::<f64>().unwrap());
        } else {
            eprintln!("Encoutered invalid float value string: {}", n_t);
            return Err(anyhow!(constants::status::INVALID_FLOAT_VALUE));
        }
    }
    Ok(tuple_vec)
}

pub fn default_vec_f64_none() -> Option<Vec<f64>> {
    None
}

pub fn default_false() -> bool {
    false
}

pub fn default_blank() -> String {
    "".to_string()
}

pub mod cahvor_format {

    use serde::{self, Deserialize, Deserializer, Serializer};

    use crate::jsonfetch::str_to_vec;
    use crate::util::string_is_valid_f64;
    use sciimg::prelude::*;

    use sciimg::vector::Vector;

    pub fn serialize<S>(model_opt: &CameraModel, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !model_opt.is_valid() {
            serializer.serialize_unit()
        } else {
            serializer.serialize_str(&model_opt.serialize())
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<CameraModel, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: Result<&str, D::Error> = Deserialize::deserialize(deserializer);
        match r {
            Err(_) => Ok(CameraModel::default()),
            Ok(s) => {
                let s0 = String::from(s);

                let split = s0.split(';');
                let mut parts: Vec<Vec<f64>> = Vec::new();

                for n in split {
                    match n.find('(') {
                        None => {
                            if string_is_valid_f64(n) {
                                parts.push(vec![n.parse::<f64>().unwrap()]);
                            }
                        }
                        Some(_i) => {
                            parts.push(str_to_vec(n).unwrap());
                        }
                    }
                }

                match parts.len() {
                    4 => {
                        // CAHV
                        Ok(CameraModel::new(Box::new(Cahv {
                            c: if !parts.is_empty() {
                                Vector::from_vec(&parts[0]).unwrap()
                            } else {
                                Vector::default()
                            },
                            a: if parts.len() >= 2 {
                                Vector::from_vec(&parts[1]).unwrap()
                            } else {
                                Vector::default()
                            },
                            h: if parts.len() >= 3 {
                                Vector::from_vec(&parts[2]).unwrap()
                            } else {
                                Vector::default()
                            },
                            v: if parts.len() >= 4 {
                                Vector::from_vec(&parts[3]).unwrap()
                            } else {
                                Vector::default()
                            },
                        })))
                    }
                    6 => {
                        // CAHVOR
                        Ok(CameraModel::new(Box::new(Cahvor {
                            c: if !parts.is_empty() {
                                Vector::from_vec(&parts[0]).unwrap()
                            } else {
                                Vector::default()
                            },
                            a: if parts.len() >= 2 {
                                Vector::from_vec(&parts[1]).unwrap()
                            } else {
                                Vector::default()
                            },
                            h: if parts.len() >= 3 {
                                Vector::from_vec(&parts[2]).unwrap()
                            } else {
                                Vector::default()
                            },
                            v: if parts.len() >= 4 {
                                Vector::from_vec(&parts[3]).unwrap()
                            } else {
                                Vector::default()
                            },
                            o: if parts.len() >= 5 {
                                Vector::from_vec(&parts[4]).unwrap()
                            } else {
                                Vector::default()
                            },
                            r: if parts.len() >= 6 {
                                Vector::from_vec(&parts[5]).unwrap()
                            } else {
                                Vector::default()
                            },
                        })))
                    }
                    9 => {
                        // CAHVORE
                        Ok(CameraModel::new(Box::new(Cahvore {
                            c: if !parts.is_empty() {
                                Vector::from_vec(&parts[0]).unwrap()
                            } else {
                                Vector::default()
                            },
                            a: if parts.len() >= 2 {
                                Vector::from_vec(&parts[1]).unwrap()
                            } else {
                                Vector::default()
                            },
                            h: if parts.len() >= 3 {
                                Vector::from_vec(&parts[2]).unwrap()
                            } else {
                                Vector::default()
                            },
                            v: if parts.len() >= 4 {
                                Vector::from_vec(&parts[3]).unwrap()
                            } else {
                                Vector::default()
                            },
                            o: if parts.len() >= 5 {
                                Vector::from_vec(&parts[4]).unwrap()
                            } else {
                                Vector::default()
                            },
                            r: if parts.len() >= 6 {
                                Vector::from_vec(&parts[5]).unwrap()
                            } else {
                                Vector::default()
                            },
                            e: if parts.len() >= 7 {
                                Vector::from_vec(&parts[6]).unwrap()
                            } else {
                                Vector::default()
                            },
                            linearity: if parts.len() >= 8 {
                                parts[7][0]
                            } else {
                                LINEARITY_PERSPECTIVE
                            },
                            pupil_type: PupilType::General,
                        })))
                    }
                    _ => Ok(CameraModel::default()),
                }
            }
        }
    }
}

pub mod tuple_format {

    use serde::{self, Deserialize, Deserializer, Serializer};

    use crate::jsonfetch::{str_to_vec, vec_to_str};

    pub fn serialize<S>(tuple_vec_opt: &Option<Vec<f64>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match tuple_vec_opt {
            None => serializer.serialize_unit(),
            Some(v) => {
                let s = vec_to_str(v);
                serializer.serialize_str(s.as_ref())
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<f64>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r: Result<&str, D::Error> = Deserialize::deserialize(deserializer);
        match r {
            Err(_) => Ok(None),
            Ok(s) => match s {
                "UNK" => Ok(None),
                _ => {
                    let tuple_vec = str_to_vec(s).unwrap();
                    Ok(Some(tuple_vec))
                }
            },
        }
    }
}
