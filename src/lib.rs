use bmp::*;
use nbdkit::*;
use std::cell::RefCell;
use std::sync::Mutex;

// BMPDisk lets you mount a BMP image file as a disk drive. Why? Because.
struct BMPDisk {
    bmpdata: Mutex<RefCell<Image>>,
}
pub fn onedee(x: u32, y: u32, width: u32) -> u64 {
    (x + (y * width)) as u64
}

pub fn twodee(offset: u64, width: u32) -> (u32, u32) {
    let x = (offset % (width as u64)) as u32;
    let y = (offset / (width as u64)) as u32;

    (x, y)
}

impl Server for BMPDisk {
    fn name() -> &'static str {
        "bmpdisk"
    }

    fn thread_model() -> Result<ThreadModel>
    where
        Self: Sized,
    {
        println!("Thread Model accessed.");
        Ok(ThreadModel::SerializeAllRequests)
    }

    fn get_size(&self) -> Result<i64> {
        println!("Size requested.");
        let width = self.bmpdata.lock().unwrap().borrow().get_width();
        let height = self.bmpdata.lock().unwrap().borrow().get_height();
        Ok((width * height) as i64)
    }

    fn open(_readonly: bool) -> Box<dyn Server>
    where
        Self: Sized,
    {
        println!("Plugin opened.");
        // Check to see if there is an existing bmp image file on disk.
        if std::path::Path::exists(std::path::Path::new("/var/tmp/bmpdisk.bmp")) {
            Box::new(BMPDisk {
                bmpdata: Mutex::new(RefCell::new(bmp::open("/var/tmp/bmpdisk.bmp").unwrap())),
            })
        } else {
            Box::new(BMPDisk {
                bmpdata: Mutex::new(RefCell::new(Image::new(4096, 4096))),
            })
        }
    }

    // OPTIMIZATION: Split across the 3 channels.

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<()> {
        println!("Read!");
        let width = self.bmpdata.lock().unwrap().borrow().get_width();
        let buf_len = buf.len();

        for idx in 0..buf_len {
            let (px, py) = twodee(idx as u64 + offset, width);
            buf[idx] = self.bmpdata.lock().unwrap().borrow().get_pixel(px, py).r;
        }

        Ok(())
    }

    fn write_at(&self, buf: &[u8], offset: u64, _flags: Flags) -> Result<()> {
        println!("Write!");
        let width = self.bmpdata.lock().unwrap().borrow().get_width();
        let buf_len = buf.len();

        for idx in 0..buf_len {
            let (px, py) = twodee(idx as u64 + offset, width);
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

        self.bmpdata
            .lock()
            .unwrap()
            .borrow()
            .save(std::path::Path::new("/var/tmp/bmpdisk.bmp"))
            .unwrap();

        Ok(())
    }

    fn can_write(&self) -> Result<bool> {
        println!("Can write accessed.");
        Ok(true)
    }
}

plugin!(BMPDisk {
    write_at,
    thread_model
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn write_new_bmp() {
        let bmpdisk = BMPDisk::open(false);
        let data: Vec<u8> = vec![0, 128, 0, 255];

        bmpdisk.write_at(&data, 0, Flags::FUA).unwrap();
    }
}
