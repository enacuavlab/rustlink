use libc::{c_void, c_char};
use std::ffi::{CString, CStr};

use std::mem;
use std::sync::Mutex;

use ivyrust::*;
use configs::RustlinkTime;



/// Structure holding data about PING time
pub struct LinkIvyPing {
	/// Time instant of sending PING message
    pub ping_instant: Mutex<RustlinkTime>,
    /// Resulting ping time [s]
    pub ping_time: f64,
    /// Filter value for EMA ping value <0,1>
    pub alpha: f64,
    /// Ping time smoothed with exponential moving
    /// average with the value of alpha [s]
    pub ping_time_ema: f64,
}


impl LinkIvyPing {
	pub fn new(alpha: f64) -> LinkIvyPing {
		LinkIvyPing {
			ping_instant: Mutex::new(RustlinkTime::new()),
			ping_time: 0.0,
			ping_time_ema: 0.0,
			alpha: alpha,
		}
	}
	
	/// Reset the underlying timer
	pub fn reset(&mut self) {
		let mut lock = self.ping_instant.lock();
	    if let Ok(ref mut ping_instant) = lock {
		    ping_instant.reset();	
	    }
	}
    
    /// Callback processing ping
    /// updates the current PING time, and resets the timer
    #[allow(dead_code)]
    pub fn callback_ping(&mut self, _: Vec<String>) {
    	let mut lock = self.ping_instant.lock();
	    if let Ok(ref mut ping_instant) = lock {
			self.ping_time = ping_instant.elapsed();
			self.ping_time_ema = self.alpha * self.ping_time + 
								 (1.0 - self.alpha) * self.ping_time;
			ping_instant.reset();
	    }
    }
    
    /// Equivalent to calling "callback_ping" but from an external source
    /// (i.e. not an Ivy bus)
    pub fn update(&mut self) {
    	let mut lock = self.ping_instant.lock();
	    if let Ok(ref mut ping_instant) = lock {
			self.ping_time = ping_instant.elapsed();
			self.ping_time_ema = self.alpha * self.ping_time + 
								 (1.0 - self.alpha) * self.ping_time_ema;
			ping_instant.reset();
	    }
    }
    
	/// Bind ivy message to a simple callback with given regexpr
	#[allow(dead_code)]
    pub fn ivy_bind_ping<F>(&mut self, cb: F, regexpr: String)
    where
        F: Fn(&mut LinkIvyPing, Vec<String>),
    {
        let regexpr = CString::new(regexpr).unwrap();
        {
	        let boxed_cb: Box<(Box<Fn(&mut LinkIvyPing, Vec<String>)>, &mut LinkIvyPing)> =
	            Box::new((Box::new(cb), self));
	        unsafe {
			        Some(IvyBindMsg(
			            apply_closure_ping,
			            Box::into_raw(boxed_cb) as *const c_void,
			            regexpr.as_ptr(),
			        ))
	        };
        }
    }
}

#[allow(dead_code)]
extern "C" fn apply_closure_ping(_app: IvyClientPtr,
                            user_data: *mut c_void,
                            argc: i32,
                            argv: *const *const c_char) {
    let mut v: Vec<String> = vec![];
    for i in 0..argc as isize {
        unsafe {
            let ptr = argv.offset(i);
            v.push(String::from(CStr::from_ptr(*ptr).to_str().unwrap()));
        }
    }
    
    let payload: &mut (Box<Fn(&mut LinkIvyPing, Vec<String>) -> ()>, &mut LinkIvyPing) =
        unsafe { mem::transmute(user_data) };
    
    payload.0(&mut payload.1, v);
}