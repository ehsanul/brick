mod bindings;
pub use bindings::*;


// Default impl for arrays only go up to 32 :(
impl Default for LiveDataPacket {
    fn default() -> LiveDataPacket {
        LiveDataPacket {
            GameCars: <[PlayerInfo; 10usize]>::default(),
            NumCars: ::std::os::raw::c_int::default(),
            GameBoosts: [BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default(), BoostInfo::default()],
            NumBoosts: ::std::os::raw::c_int::default(),
            GameBall: BallInfo::default(),
            GameInfo: GameInfo::default(),
        }
    }
}
