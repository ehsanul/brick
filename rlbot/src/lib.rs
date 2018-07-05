#[macro_use]
extern crate lazy_static;

extern crate libloading as lib;

mod bindings;
pub use bindings::*;


#[repr(C)]
pub enum RLBotCoreStatus {
	Success,
	BufferOverfilled,
	MessageLargerThanMax,
	InvalidNumPlayers,
	InvalidBotSkill,
	InvalidHumanIndex,
	InvalidName,
	InvalidTeam,
	InvalidTeamColorID,
	InvalidCustomColorID,
	InvalidGameValues,
	InvalidThrottle,
	InvalidSteer,
	InvalidPitch,
	InvalidYaw,
	InvalidRoll,
	InvalidPlayerIndex,
	InvalidQuickChatPreset,
	InvalidRenderType
}


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


lazy_static! {
    static ref RLBOT_INTERFACE: lib::Library = {
        lib::Library::new("RLBot_Core_Interface.dll").expect("Couldn't find RLBot_Core_Interface.dll")
    };
}


// DLL_EXPORT RLBotCoreStatus RLBOT_CORE_API UpdateLiveDataPacket(LiveDataPacket* pLiveData);
type UpdateLiveDataPacketFunc = unsafe extern fn(&mut LiveDataPacket) -> RLBotCoreStatus;
pub fn update_live_data_packet(packet: &mut LiveDataPacket) -> Result<(), RLBotCoreStatus> {
    unsafe {
        // TODO cache func
        let func = RLBOT_INTERFACE.get::<UpdateLiveDataPacketFunc>(b"UpdateLiveDataPacket").expect("Couldn't find UpdateLiveDataPacket");
        match func(packet) {
            RLBotCoreStatus::Success => Ok(()),
            e => Err(e),
        }
    }
}

// DLL_EXPORT RLBotCoreStatus RLBOT_CORE_API UpdatePlayerInput(PlayerInput playerInput, int playerIndex);
type UpdatePlayerInputFunc = unsafe extern fn(PlayerInput, ::std::os::raw::c_int) -> RLBotCoreStatus;
pub fn update_player_input(player_input: PlayerInput, player_index: i32) -> Result<(), RLBotCoreStatus> {
    unsafe {
        // TODO cache func
        let func = RLBOT_INTERFACE.get::<UpdatePlayerInputFunc>(b"UpdatePlayerInput").expect("Couldn't find UpdatePlayerInput");
        match func(player_input, player_index) {
            RLBotCoreStatus::Success => Ok(()),
            e => Err(e),
        }
    }
}