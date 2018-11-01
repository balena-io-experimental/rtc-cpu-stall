#[macro_use]
extern crate nix;

use std::fs::File;

use std::os::raw::c_int;
use std::os::unix::io::AsRawFd;

use std::io;
use std::io::Read;
use std::io::Write;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct RtcTime {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
}

// From `linux/rtc.h`
ioctl_none!(rtc_uie_on, 'p', 0x03);
ioctl_none!(rtc_uie_off, 'p', 0x04);
ioctl_read!(rtc_rd_time, 'p', 0x09, RtcTime);
ioctl_write_ptr!(rtc_set_time, 'p', 0x0a, RtcTime);

#[derive(Debug)]
pub struct RtcDev {
    clock: File,
}

impl RtcDev {
    pub fn open(dev: &str) -> RtcDev {
        RtcDev {
            clock: File::open(dev).expect("Opening RTC failed"),
        }
    }

    pub fn get_time(&self) -> RtcTime {
        let mut time = RtcTime::default();

        assert_eq!(
            0,
            unsafe { rtc_rd_time(self.clock.as_raw_fd(), &mut time) }.expect("Get time failed")
        );

        time
    }

    pub fn set_time(&self, time: &RtcTime) {
        assert_eq!(
            0,
            unsafe { rtc_set_time(self.clock.as_raw_fd(), time) }.expect("Set time failed")
        );
    }

    pub fn update_interrupt_enable(&self) {
        assert_eq!(
            0,
            unsafe { rtc_uie_on(self.clock.as_raw_fd()) }.expect("Update interrupt enable failed")
        );
    }

    pub fn update_interrupt_disable(&self) {
        assert_eq!(
            0,
            unsafe { rtc_uie_off(self.clock.as_raw_fd()) }
                .expect("Update interrupt disable failed")
        );
    }

    pub fn read_data(&mut self, data: &mut [u8]) {
        self.clock.read_exact(data).expect("Reading data failed");
    }
}

fn main() {
    let mut rtc = RtcDev::open("/dev/rtc0");

    let mut time = rtc.get_time();

    time.tm_year = 70;
    rtc.set_time(&time);

    rtc.update_interrupt_enable();

    println!(
        "Counting 5 update (1/sec) interrupts from reading {:?}",
        rtc
    );
    io::stdout().flush().expect("Flushing stdout failed");

    for i in 1..6 {
        if i == 3 {
            time.tm_year = 130;
            rtc.set_time(&time);
        }

        let mut data = [0; 4];

        rtc.read_data(&mut data);

        println!("{} {:?}", i, data);
        io::stdout().flush().expect("Flushing stdout failed");
    }

    rtc.update_interrupt_disable();

    println!("{:?}", time);
}
