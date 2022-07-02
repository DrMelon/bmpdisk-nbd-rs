use bmp::*;
use fast_hilbert::*;
use lazy_static::lazy_static;
use nbdkit::*;
use std::cell::RefCell;
use std::sync::Mutex;

// BMPDisk lets you mount a BMP image file as a disk drive. Why? Because.
struct BMPDisk {
    bmpdata: Mutex<RefCell<Image>>,
}

struct BMPConfig {
    filename: String,
    split_channels: bool,
    dimensions: u32,
}

lazy_static! {
    static ref CONFIG: Mutex<BMPConfig> = Mutex::new(BMPConfig {
        filename: "/var/tmp/bmpdisk.bmp".to_string(),
        dimensions: 4096,
        split_channels: true
    });
}

impl Server for BMPDisk {
    fn name() -> &'static str {
        "bmpdisk"
    }

    fn thread_model() -> Result<ThreadModel>
    where
        Self: Sized,
    {
        Ok(ThreadModel::SerializeAllRequests)
    }

    fn get_size(&self) -> Result<i64> {
        let width = self.bmpdata.lock().unwrap().borrow().get_width();
        let height = self.bmpdata.lock().unwrap().borrow().get_height();

        let size = if CONFIG.lock().unwrap().split_channels {
            width * height * 3
        } else {
            width * height
        };
        Ok(size as i64)
    }

    fn open(_readonly: bool) -> Box<dyn Server>
    where
        Self: Sized,
    {
        // Check to see if there is an existing bmp image file on disk.
        if std::path::Path::exists(std::path::Path::new(&CONFIG.lock().unwrap().filename[..])) {
            Box::new(BMPDisk {
                bmpdata: Mutex::new(RefCell::new(
                    bmp::open(CONFIG.lock().unwrap().filename.to_string()).unwrap(),
                )),
            })
        } else {
            let dimensions = CONFIG.lock().unwrap().dimensions;
            Box::new(BMPDisk {
                bmpdata: Mutex::new(RefCell::new(Image::new(dimensions, dimensions))),
            })
        }
    }

    // OPTIMIZATION: Split across the 3 channels.

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<()> {
        let width = self.bmpdata.lock().unwrap().borrow().get_width();
        let buf_len = buf.len();

        if !CONFIG.lock().unwrap().split_channels {
            for idx in 0..buf_len {
                let (px, py) = h2xy(idx as u64);
                buf[idx] = self.bmpdata.lock().unwrap().borrow().get_pixel(px, py).r;
            }
        } else {
            for idx in 0..buf_len {
                let adjusted_idx = idx as u64 + offset;
                let total_dimensions = width * width;
                let page = adjusted_idx / total_dimensions as u64;
                let normalized_idx = adjusted_idx % total_dimensions as u64;
                let (px, py) = h2xy(normalized_idx);
                let pixel = self.bmpdata.lock().unwrap().borrow().get_pixel(px, py);
                buf[idx] = match page {
                    0 => pixel.r,
                    1 => pixel.g,
                    2 => pixel.b,
                    _ => panic!("Page index out of range!"),
                }
            }
        }

        Ok(())
    }

    fn write_at(&self, buf: &[u8], offset: u64, _flags: Flags) -> Result<()> {
        let width = self.bmpdata.lock().unwrap().borrow().get_width();
        let buf_len = buf.len();

        if !CONFIG.lock().unwrap().split_channels {
            for idx in 0..buf_len {
                let (px, py) = h2xy(idx as u64);
                let pxdata = Pixel {
                    r: buf[idx],
                    g: buf[idx],
                    b: buf[idx],
                };
                self.bmpdata
                    .lock()
                    .unwrap()
                    .borrow_mut()
                    .set_pixel(px, py, pxdata);
            }
        } else {
            for idx in 0..buf_len {
                let adjusted_idx = idx as u64 + offset;
                let total_dimensions = width * width;
                let page = adjusted_idx / total_dimensions as u64;
                let normalized_idx = adjusted_idx % total_dimensions as u64;
                let (px, py) = h2xy(normalized_idx);
                let mut pxdata = self.bmpdata.lock().unwrap().borrow().get_pixel(px, py);
                match page {
                    0 => pxdata.r = buf[idx],
                    1 => pxdata.g = buf[idx],
                    2 => pxdata.b = buf[idx],
                    _ => panic!("Page index out of range!"),
                };
                self.bmpdata
                    .lock()
                    .unwrap()
                    .borrow_mut()
                    .set_pixel(px, py, pxdata);
            }
        }

        self.bmpdata
            .lock()
            .unwrap()
            .borrow()
            .save(std::path::Path::new(&CONFIG.lock().unwrap().filename[..]))
            .unwrap();

        Ok(())
    }

    fn config(key: &str, value: &str) -> Result<()>
    where
        Self: Sized,
    {
        match key {
            "filename" => CONFIG.lock().unwrap().filename = value.to_string(),
            "no-split-channels" => CONFIG.lock().unwrap().split_channels = false,
            "dimensions" => {
                CONFIG.lock().unwrap().dimensions = value.to_string().parse::<u32>().unwrap()
            }
            _ => {}
        };
        Ok(())
    }
}

plugin!(BMPDisk {
    write_at,
    thread_model,
    config
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn write_new_bmp() {
        let bmpdisk = BMPDisk::open(false);
        let data: Vec<u8> = vec![0, 128, 0, 255];

        assert!(bmpdisk.write_at(&data, 0, Flags::FUA).is_ok());
    }
}
