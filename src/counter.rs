
pub struct CountDown{
    reset_time: u32,
    pub time_left: u32
}
impl CountDown{
    pub fn new(start_time: u32) -> CountDown{
        CountDown{reset_time: start_time, time_left: start_time}
    }

    pub fn tick(&mut self){
        if self.time_left == 0{
            self.time_left+=1;
        }
        self.time_left -=1;
    }

    pub fn reset(&mut self){
        self.time_left = self.reset_time.clone();
    }

    pub fn has_time_left(&self) -> bool{
        self.time_left>0
    }
}