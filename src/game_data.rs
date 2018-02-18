// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct ControllerState {
    // message fields
    pub throttle: f32,
    pub steer: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub jump: bool,
    pub boost: bool,
    pub handbrake: bool,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ControllerState {}

impl ControllerState {
    pub fn new() -> ControllerState {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ControllerState {
        static mut instance: ::protobuf::lazy::Lazy<ControllerState> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ControllerState,
        };
        unsafe {
            instance.get(ControllerState::new)
        }
    }

    // float throttle = 1;

    pub fn clear_throttle(&mut self) {
        self.throttle = 0.;
    }

    // Param is passed by value, moved
    pub fn set_throttle(&mut self, v: f32) {
        self.throttle = v;
    }

    pub fn get_throttle(&self) -> f32 {
        self.throttle
    }

    fn get_throttle_for_reflect(&self) -> &f32 {
        &self.throttle
    }

    fn mut_throttle_for_reflect(&mut self) -> &mut f32 {
        &mut self.throttle
    }

    // float steer = 2;

    pub fn clear_steer(&mut self) {
        self.steer = 0.;
    }

    // Param is passed by value, moved
    pub fn set_steer(&mut self, v: f32) {
        self.steer = v;
    }

    pub fn get_steer(&self) -> f32 {
        self.steer
    }

    fn get_steer_for_reflect(&self) -> &f32 {
        &self.steer
    }

    fn mut_steer_for_reflect(&mut self) -> &mut f32 {
        &mut self.steer
    }

    // float pitch = 3;

    pub fn clear_pitch(&mut self) {
        self.pitch = 0.;
    }

    // Param is passed by value, moved
    pub fn set_pitch(&mut self, v: f32) {
        self.pitch = v;
    }

    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    fn get_pitch_for_reflect(&self) -> &f32 {
        &self.pitch
    }

    fn mut_pitch_for_reflect(&mut self) -> &mut f32 {
        &mut self.pitch
    }

    // float yaw = 4;

    pub fn clear_yaw(&mut self) {
        self.yaw = 0.;
    }

    // Param is passed by value, moved
    pub fn set_yaw(&mut self, v: f32) {
        self.yaw = v;
    }

    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }

    fn get_yaw_for_reflect(&self) -> &f32 {
        &self.yaw
    }

    fn mut_yaw_for_reflect(&mut self) -> &mut f32 {
        &mut self.yaw
    }

    // float roll = 5;

    pub fn clear_roll(&mut self) {
        self.roll = 0.;
    }

    // Param is passed by value, moved
    pub fn set_roll(&mut self, v: f32) {
        self.roll = v;
    }

    pub fn get_roll(&self) -> f32 {
        self.roll
    }

    fn get_roll_for_reflect(&self) -> &f32 {
        &self.roll
    }

    fn mut_roll_for_reflect(&mut self) -> &mut f32 {
        &mut self.roll
    }

    // bool jump = 6;

    pub fn clear_jump(&mut self) {
        self.jump = false;
    }

    // Param is passed by value, moved
    pub fn set_jump(&mut self, v: bool) {
        self.jump = v;
    }

    pub fn get_jump(&self) -> bool {
        self.jump
    }

    fn get_jump_for_reflect(&self) -> &bool {
        &self.jump
    }

    fn mut_jump_for_reflect(&mut self) -> &mut bool {
        &mut self.jump
    }

    // bool boost = 7;

    pub fn clear_boost(&mut self) {
        self.boost = false;
    }

    // Param is passed by value, moved
    pub fn set_boost(&mut self, v: bool) {
        self.boost = v;
    }

    pub fn get_boost(&self) -> bool {
        self.boost
    }

    fn get_boost_for_reflect(&self) -> &bool {
        &self.boost
    }

    fn mut_boost_for_reflect(&mut self) -> &mut bool {
        &mut self.boost
    }

    // bool handbrake = 8;

    pub fn clear_handbrake(&mut self) {
        self.handbrake = false;
    }

    // Param is passed by value, moved
    pub fn set_handbrake(&mut self, v: bool) {
        self.handbrake = v;
    }

    pub fn get_handbrake(&self) -> bool {
        self.handbrake
    }

    fn get_handbrake_for_reflect(&self) -> &bool {
        &self.handbrake
    }

    fn mut_handbrake_for_reflect(&mut self) -> &mut bool {
        &mut self.handbrake
    }
}

impl ::protobuf::Message for ControllerState {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.throttle = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.steer = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.pitch = tmp;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.yaw = tmp;
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.roll = tmp;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.jump = tmp;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.boost = tmp;
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.handbrake = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.throttle != 0. {
            my_size += 5;
        }
        if self.steer != 0. {
            my_size += 5;
        }
        if self.pitch != 0. {
            my_size += 5;
        }
        if self.yaw != 0. {
            my_size += 5;
        }
        if self.roll != 0. {
            my_size += 5;
        }
        if self.jump != false {
            my_size += 2;
        }
        if self.boost != false {
            my_size += 2;
        }
        if self.handbrake != false {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.throttle != 0. {
            os.write_float(1, self.throttle)?;
        }
        if self.steer != 0. {
            os.write_float(2, self.steer)?;
        }
        if self.pitch != 0. {
            os.write_float(3, self.pitch)?;
        }
        if self.yaw != 0. {
            os.write_float(4, self.yaw)?;
        }
        if self.roll != 0. {
            os.write_float(5, self.roll)?;
        }
        if self.jump != false {
            os.write_bool(6, self.jump)?;
        }
        if self.boost != false {
            os.write_bool(7, self.boost)?;
        }
        if self.handbrake != false {
            os.write_bool(8, self.handbrake)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ControllerState {
    fn new() -> ControllerState {
        ControllerState::new()
    }

    fn descriptor_static(_: ::std::option::Option<ControllerState>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "throttle",
                    ControllerState::get_throttle_for_reflect,
                    ControllerState::mut_throttle_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "steer",
                    ControllerState::get_steer_for_reflect,
                    ControllerState::mut_steer_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "pitch",
                    ControllerState::get_pitch_for_reflect,
                    ControllerState::mut_pitch_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "yaw",
                    ControllerState::get_yaw_for_reflect,
                    ControllerState::mut_yaw_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "roll",
                    ControllerState::get_roll_for_reflect,
                    ControllerState::mut_roll_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "jump",
                    ControllerState::get_jump_for_reflect,
                    ControllerState::mut_jump_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "boost",
                    ControllerState::get_boost_for_reflect,
                    ControllerState::mut_boost_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "handbrake",
                    ControllerState::get_handbrake_for_reflect,
                    ControllerState::mut_handbrake_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ControllerState>(
                    "ControllerState",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ControllerState {
    fn clear(&mut self) {
        self.clear_throttle();
        self.clear_steer();
        self.clear_pitch();
        self.clear_yaw();
        self.clear_roll();
        self.clear_jump();
        self.clear_boost();
        self.clear_handbrake();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ControllerState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ControllerState {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Vector3 {
    // message fields
    pub x: f32,
    pub y: f32,
    pub z: f32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Vector3 {}

impl Vector3 {
    pub fn new() -> Vector3 {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Vector3 {
        static mut instance: ::protobuf::lazy::Lazy<Vector3> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Vector3,
        };
        unsafe {
            instance.get(Vector3::new)
        }
    }

    // float x = 1;

    pub fn clear_x(&mut self) {
        self.x = 0.;
    }

    // Param is passed by value, moved
    pub fn set_x(&mut self, v: f32) {
        self.x = v;
    }

    pub fn get_x(&self) -> f32 {
        self.x
    }

    fn get_x_for_reflect(&self) -> &f32 {
        &self.x
    }

    fn mut_x_for_reflect(&mut self) -> &mut f32 {
        &mut self.x
    }

    // float y = 2;

    pub fn clear_y(&mut self) {
        self.y = 0.;
    }

    // Param is passed by value, moved
    pub fn set_y(&mut self, v: f32) {
        self.y = v;
    }

    pub fn get_y(&self) -> f32 {
        self.y
    }

    fn get_y_for_reflect(&self) -> &f32 {
        &self.y
    }

    fn mut_y_for_reflect(&mut self) -> &mut f32 {
        &mut self.y
    }

    // float z = 3;

    pub fn clear_z(&mut self) {
        self.z = 0.;
    }

    // Param is passed by value, moved
    pub fn set_z(&mut self, v: f32) {
        self.z = v;
    }

    pub fn get_z(&self) -> f32 {
        self.z
    }

    fn get_z_for_reflect(&self) -> &f32 {
        &self.z
    }

    fn mut_z_for_reflect(&mut self) -> &mut f32 {
        &mut self.z
    }
}

impl ::protobuf::Message for Vector3 {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.x = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.y = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.z = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.x != 0. {
            my_size += 5;
        }
        if self.y != 0. {
            my_size += 5;
        }
        if self.z != 0. {
            my_size += 5;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.x != 0. {
            os.write_float(1, self.x)?;
        }
        if self.y != 0. {
            os.write_float(2, self.y)?;
        }
        if self.z != 0. {
            os.write_float(3, self.z)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Vector3 {
    fn new() -> Vector3 {
        Vector3::new()
    }

    fn descriptor_static(_: ::std::option::Option<Vector3>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "x",
                    Vector3::get_x_for_reflect,
                    Vector3::mut_x_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "y",
                    Vector3::get_y_for_reflect,
                    Vector3::mut_y_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "z",
                    Vector3::get_z_for_reflect,
                    Vector3::mut_z_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Vector3>(
                    "Vector3",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Vector3 {
    fn clear(&mut self) {
        self.clear_x();
        self.clear_y();
        self.clear_z();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Vector3 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Vector3 {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Rotator {
    // message fields
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Rotator {}

impl Rotator {
    pub fn new() -> Rotator {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Rotator {
        static mut instance: ::protobuf::lazy::Lazy<Rotator> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Rotator,
        };
        unsafe {
            instance.get(Rotator::new)
        }
    }

    // float pitch = 1;

    pub fn clear_pitch(&mut self) {
        self.pitch = 0.;
    }

    // Param is passed by value, moved
    pub fn set_pitch(&mut self, v: f32) {
        self.pitch = v;
    }

    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    fn get_pitch_for_reflect(&self) -> &f32 {
        &self.pitch
    }

    fn mut_pitch_for_reflect(&mut self) -> &mut f32 {
        &mut self.pitch
    }

    // float yaw = 2;

    pub fn clear_yaw(&mut self) {
        self.yaw = 0.;
    }

    // Param is passed by value, moved
    pub fn set_yaw(&mut self, v: f32) {
        self.yaw = v;
    }

    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }

    fn get_yaw_for_reflect(&self) -> &f32 {
        &self.yaw
    }

    fn mut_yaw_for_reflect(&mut self) -> &mut f32 {
        &mut self.yaw
    }

    // float roll = 3;

    pub fn clear_roll(&mut self) {
        self.roll = 0.;
    }

    // Param is passed by value, moved
    pub fn set_roll(&mut self, v: f32) {
        self.roll = v;
    }

    pub fn get_roll(&self) -> f32 {
        self.roll
    }

    fn get_roll_for_reflect(&self) -> &f32 {
        &self.roll
    }

    fn mut_roll_for_reflect(&mut self) -> &mut f32 {
        &mut self.roll
    }
}

impl ::protobuf::Message for Rotator {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.pitch = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.yaw = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.roll = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.pitch != 0. {
            my_size += 5;
        }
        if self.yaw != 0. {
            my_size += 5;
        }
        if self.roll != 0. {
            my_size += 5;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.pitch != 0. {
            os.write_float(1, self.pitch)?;
        }
        if self.yaw != 0. {
            os.write_float(2, self.yaw)?;
        }
        if self.roll != 0. {
            os.write_float(3, self.roll)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Rotator {
    fn new() -> Rotator {
        Rotator::new()
    }

    fn descriptor_static(_: ::std::option::Option<Rotator>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "pitch",
                    Rotator::get_pitch_for_reflect,
                    Rotator::mut_pitch_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "yaw",
                    Rotator::get_yaw_for_reflect,
                    Rotator::mut_yaw_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "roll",
                    Rotator::get_roll_for_reflect,
                    Rotator::mut_roll_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Rotator>(
                    "Rotator",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Rotator {
    fn clear(&mut self) {
        self.clear_pitch();
        self.clear_yaw();
        self.clear_roll();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Rotator {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Rotator {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Touch {
    // message fields
    pub player_name: ::std::string::String,
    pub game_seconds: f32,
    pub location: ::protobuf::SingularPtrField<Vector3>,
    pub normal: ::protobuf::SingularPtrField<Vector3>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Touch {}

impl Touch {
    pub fn new() -> Touch {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Touch {
        static mut instance: ::protobuf::lazy::Lazy<Touch> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Touch,
        };
        unsafe {
            instance.get(Touch::new)
        }
    }

    // string player_name = 1;

    pub fn clear_player_name(&mut self) {
        self.player_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_player_name(&mut self, v: ::std::string::String) {
        self.player_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_player_name(&mut self) -> &mut ::std::string::String {
        &mut self.player_name
    }

    // Take field
    pub fn take_player_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.player_name, ::std::string::String::new())
    }

    pub fn get_player_name(&self) -> &str {
        &self.player_name
    }

    fn get_player_name_for_reflect(&self) -> &::std::string::String {
        &self.player_name
    }

    fn mut_player_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.player_name
    }

    // float game_seconds = 2;

    pub fn clear_game_seconds(&mut self) {
        self.game_seconds = 0.;
    }

    // Param is passed by value, moved
    pub fn set_game_seconds(&mut self, v: f32) {
        self.game_seconds = v;
    }

    pub fn get_game_seconds(&self) -> f32 {
        self.game_seconds
    }

    fn get_game_seconds_for_reflect(&self) -> &f32 {
        &self.game_seconds
    }

    fn mut_game_seconds_for_reflect(&mut self) -> &mut f32 {
        &mut self.game_seconds
    }

    // .rlbot.api.Vector3 location = 3;

    pub fn clear_location(&mut self) {
        self.location.clear();
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    // Param is passed by value, moved
    pub fn set_location(&mut self, v: Vector3) {
        self.location = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_location(&mut self) -> &mut Vector3 {
        if self.location.is_none() {
            self.location.set_default();
        }
        self.location.as_mut().unwrap()
    }

    // Take field
    pub fn take_location(&mut self) -> Vector3 {
        self.location.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_location(&self) -> &Vector3 {
        self.location.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_location_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.location
    }

    fn mut_location_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.location
    }

    // .rlbot.api.Vector3 normal = 4;

    pub fn clear_normal(&mut self) {
        self.normal.clear();
    }

    pub fn has_normal(&self) -> bool {
        self.normal.is_some()
    }

    // Param is passed by value, moved
    pub fn set_normal(&mut self, v: Vector3) {
        self.normal = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_normal(&mut self) -> &mut Vector3 {
        if self.normal.is_none() {
            self.normal.set_default();
        }
        self.normal.as_mut().unwrap()
    }

    // Take field
    pub fn take_normal(&mut self) -> Vector3 {
        self.normal.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_normal(&self) -> &Vector3 {
        self.normal.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_normal_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.normal
    }

    fn mut_normal_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.normal
    }
}

impl ::protobuf::Message for Touch {
    fn is_initialized(&self) -> bool {
        for v in &self.location {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.normal {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.player_name)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.game_seconds = tmp;
                },
                3 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.location)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.normal)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.player_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.player_name);
        }
        if self.game_seconds != 0. {
            my_size += 5;
        }
        if let Some(ref v) = self.location.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.normal.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.player_name.is_empty() {
            os.write_string(1, &self.player_name)?;
        }
        if self.game_seconds != 0. {
            os.write_float(2, self.game_seconds)?;
        }
        if let Some(ref v) = self.location.as_ref() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.normal.as_ref() {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Touch {
    fn new() -> Touch {
        Touch::new()
    }

    fn descriptor_static(_: ::std::option::Option<Touch>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "player_name",
                    Touch::get_player_name_for_reflect,
                    Touch::mut_player_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "game_seconds",
                    Touch::get_game_seconds_for_reflect,
                    Touch::mut_game_seconds_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "location",
                    Touch::get_location_for_reflect,
                    Touch::mut_location_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "normal",
                    Touch::get_normal_for_reflect,
                    Touch::mut_normal_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Touch>(
                    "Touch",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Touch {
    fn clear(&mut self) {
        self.clear_player_name();
        self.clear_game_seconds();
        self.clear_location();
        self.clear_normal();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Touch {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Touch {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ScoreInfo {
    // message fields
    pub score: i32,
    pub goals: i32,
    pub own_goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub demolitions: i32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ScoreInfo {}

impl ScoreInfo {
    pub fn new() -> ScoreInfo {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ScoreInfo {
        static mut instance: ::protobuf::lazy::Lazy<ScoreInfo> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ScoreInfo,
        };
        unsafe {
            instance.get(ScoreInfo::new)
        }
    }

    // int32 score = 1;

    pub fn clear_score(&mut self) {
        self.score = 0;
    }

    // Param is passed by value, moved
    pub fn set_score(&mut self, v: i32) {
        self.score = v;
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    fn get_score_for_reflect(&self) -> &i32 {
        &self.score
    }

    fn mut_score_for_reflect(&mut self) -> &mut i32 {
        &mut self.score
    }

    // int32 goals = 2;

    pub fn clear_goals(&mut self) {
        self.goals = 0;
    }

    // Param is passed by value, moved
    pub fn set_goals(&mut self, v: i32) {
        self.goals = v;
    }

    pub fn get_goals(&self) -> i32 {
        self.goals
    }

    fn get_goals_for_reflect(&self) -> &i32 {
        &self.goals
    }

    fn mut_goals_for_reflect(&mut self) -> &mut i32 {
        &mut self.goals
    }

    // int32 own_goals = 3;

    pub fn clear_own_goals(&mut self) {
        self.own_goals = 0;
    }

    // Param is passed by value, moved
    pub fn set_own_goals(&mut self, v: i32) {
        self.own_goals = v;
    }

    pub fn get_own_goals(&self) -> i32 {
        self.own_goals
    }

    fn get_own_goals_for_reflect(&self) -> &i32 {
        &self.own_goals
    }

    fn mut_own_goals_for_reflect(&mut self) -> &mut i32 {
        &mut self.own_goals
    }

    // int32 assists = 4;

    pub fn clear_assists(&mut self) {
        self.assists = 0;
    }

    // Param is passed by value, moved
    pub fn set_assists(&mut self, v: i32) {
        self.assists = v;
    }

    pub fn get_assists(&self) -> i32 {
        self.assists
    }

    fn get_assists_for_reflect(&self) -> &i32 {
        &self.assists
    }

    fn mut_assists_for_reflect(&mut self) -> &mut i32 {
        &mut self.assists
    }

    // int32 saves = 5;

    pub fn clear_saves(&mut self) {
        self.saves = 0;
    }

    // Param is passed by value, moved
    pub fn set_saves(&mut self, v: i32) {
        self.saves = v;
    }

    pub fn get_saves(&self) -> i32 {
        self.saves
    }

    fn get_saves_for_reflect(&self) -> &i32 {
        &self.saves
    }

    fn mut_saves_for_reflect(&mut self) -> &mut i32 {
        &mut self.saves
    }

    // int32 shots = 6;

    pub fn clear_shots(&mut self) {
        self.shots = 0;
    }

    // Param is passed by value, moved
    pub fn set_shots(&mut self, v: i32) {
        self.shots = v;
    }

    pub fn get_shots(&self) -> i32 {
        self.shots
    }

    fn get_shots_for_reflect(&self) -> &i32 {
        &self.shots
    }

    fn mut_shots_for_reflect(&mut self) -> &mut i32 {
        &mut self.shots
    }

    // int32 demolitions = 7;

    pub fn clear_demolitions(&mut self) {
        self.demolitions = 0;
    }

    // Param is passed by value, moved
    pub fn set_demolitions(&mut self, v: i32) {
        self.demolitions = v;
    }

    pub fn get_demolitions(&self) -> i32 {
        self.demolitions
    }

    fn get_demolitions_for_reflect(&self) -> &i32 {
        &self.demolitions
    }

    fn mut_demolitions_for_reflect(&mut self) -> &mut i32 {
        &mut self.demolitions
    }
}

impl ::protobuf::Message for ScoreInfo {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.score = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.goals = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.own_goals = tmp;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.assists = tmp;
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.saves = tmp;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.shots = tmp;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.demolitions = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.score != 0 {
            my_size += ::protobuf::rt::value_size(1, self.score, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.goals != 0 {
            my_size += ::protobuf::rt::value_size(2, self.goals, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.own_goals != 0 {
            my_size += ::protobuf::rt::value_size(3, self.own_goals, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.assists != 0 {
            my_size += ::protobuf::rt::value_size(4, self.assists, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.saves != 0 {
            my_size += ::protobuf::rt::value_size(5, self.saves, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.shots != 0 {
            my_size += ::protobuf::rt::value_size(6, self.shots, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.demolitions != 0 {
            my_size += ::protobuf::rt::value_size(7, self.demolitions, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.score != 0 {
            os.write_int32(1, self.score)?;
        }
        if self.goals != 0 {
            os.write_int32(2, self.goals)?;
        }
        if self.own_goals != 0 {
            os.write_int32(3, self.own_goals)?;
        }
        if self.assists != 0 {
            os.write_int32(4, self.assists)?;
        }
        if self.saves != 0 {
            os.write_int32(5, self.saves)?;
        }
        if self.shots != 0 {
            os.write_int32(6, self.shots)?;
        }
        if self.demolitions != 0 {
            os.write_int32(7, self.demolitions)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ScoreInfo {
    fn new() -> ScoreInfo {
        ScoreInfo::new()
    }

    fn descriptor_static(_: ::std::option::Option<ScoreInfo>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "score",
                    ScoreInfo::get_score_for_reflect,
                    ScoreInfo::mut_score_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "goals",
                    ScoreInfo::get_goals_for_reflect,
                    ScoreInfo::mut_goals_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "own_goals",
                    ScoreInfo::get_own_goals_for_reflect,
                    ScoreInfo::mut_own_goals_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "assists",
                    ScoreInfo::get_assists_for_reflect,
                    ScoreInfo::mut_assists_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "saves",
                    ScoreInfo::get_saves_for_reflect,
                    ScoreInfo::mut_saves_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "shots",
                    ScoreInfo::get_shots_for_reflect,
                    ScoreInfo::mut_shots_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "demolitions",
                    ScoreInfo::get_demolitions_for_reflect,
                    ScoreInfo::mut_demolitions_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ScoreInfo>(
                    "ScoreInfo",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ScoreInfo {
    fn clear(&mut self) {
        self.clear_score();
        self.clear_goals();
        self.clear_own_goals();
        self.clear_assists();
        self.clear_saves();
        self.clear_shots();
        self.clear_demolitions();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ScoreInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ScoreInfo {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct PlayerInfo {
    // message fields
    pub location: ::protobuf::SingularPtrField<Vector3>,
    pub rotation: ::protobuf::SingularPtrField<Rotator>,
    pub velocity: ::protobuf::SingularPtrField<Vector3>,
    pub angular_velocity: ::protobuf::SingularPtrField<Vector3>,
    pub score_info: ::protobuf::SingularPtrField<ScoreInfo>,
    pub is_demolished: bool,
    pub is_midair: bool,
    pub is_supersonic: bool,
    pub is_bot: bool,
    pub jumped: bool,
    pub double_jumped: bool,
    pub name: ::std::string::String,
    pub team: i32,
    pub boost: i32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for PlayerInfo {}

impl PlayerInfo {
    pub fn new() -> PlayerInfo {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static PlayerInfo {
        static mut instance: ::protobuf::lazy::Lazy<PlayerInfo> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const PlayerInfo,
        };
        unsafe {
            instance.get(PlayerInfo::new)
        }
    }

    // .rlbot.api.Vector3 location = 1;

    pub fn clear_location(&mut self) {
        self.location.clear();
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    // Param is passed by value, moved
    pub fn set_location(&mut self, v: Vector3) {
        self.location = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_location(&mut self) -> &mut Vector3 {
        if self.location.is_none() {
            self.location.set_default();
        }
        self.location.as_mut().unwrap()
    }

    // Take field
    pub fn take_location(&mut self) -> Vector3 {
        self.location.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_location(&self) -> &Vector3 {
        self.location.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_location_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.location
    }

    fn mut_location_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.location
    }

    // .rlbot.api.Rotator rotation = 2;

    pub fn clear_rotation(&mut self) {
        self.rotation.clear();
    }

    pub fn has_rotation(&self) -> bool {
        self.rotation.is_some()
    }

    // Param is passed by value, moved
    pub fn set_rotation(&mut self, v: Rotator) {
        self.rotation = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_rotation(&mut self) -> &mut Rotator {
        if self.rotation.is_none() {
            self.rotation.set_default();
        }
        self.rotation.as_mut().unwrap()
    }

    // Take field
    pub fn take_rotation(&mut self) -> Rotator {
        self.rotation.take().unwrap_or_else(|| Rotator::new())
    }

    pub fn get_rotation(&self) -> &Rotator {
        self.rotation.as_ref().unwrap_or_else(|| Rotator::default_instance())
    }

    fn get_rotation_for_reflect(&self) -> &::protobuf::SingularPtrField<Rotator> {
        &self.rotation
    }

    fn mut_rotation_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Rotator> {
        &mut self.rotation
    }

    // .rlbot.api.Vector3 velocity = 3;

    pub fn clear_velocity(&mut self) {
        self.velocity.clear();
    }

    pub fn has_velocity(&self) -> bool {
        self.velocity.is_some()
    }

    // Param is passed by value, moved
    pub fn set_velocity(&mut self, v: Vector3) {
        self.velocity = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_velocity(&mut self) -> &mut Vector3 {
        if self.velocity.is_none() {
            self.velocity.set_default();
        }
        self.velocity.as_mut().unwrap()
    }

    // Take field
    pub fn take_velocity(&mut self) -> Vector3 {
        self.velocity.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_velocity(&self) -> &Vector3 {
        self.velocity.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_velocity_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.velocity
    }

    fn mut_velocity_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.velocity
    }

    // .rlbot.api.Vector3 angular_velocity = 4;

    pub fn clear_angular_velocity(&mut self) {
        self.angular_velocity.clear();
    }

    pub fn has_angular_velocity(&self) -> bool {
        self.angular_velocity.is_some()
    }

    // Param is passed by value, moved
    pub fn set_angular_velocity(&mut self, v: Vector3) {
        self.angular_velocity = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_angular_velocity(&mut self) -> &mut Vector3 {
        if self.angular_velocity.is_none() {
            self.angular_velocity.set_default();
        }
        self.angular_velocity.as_mut().unwrap()
    }

    // Take field
    pub fn take_angular_velocity(&mut self) -> Vector3 {
        self.angular_velocity.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_angular_velocity(&self) -> &Vector3 {
        self.angular_velocity.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_angular_velocity_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.angular_velocity
    }

    fn mut_angular_velocity_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.angular_velocity
    }

    // .rlbot.api.ScoreInfo score_info = 5;

    pub fn clear_score_info(&mut self) {
        self.score_info.clear();
    }

    pub fn has_score_info(&self) -> bool {
        self.score_info.is_some()
    }

    // Param is passed by value, moved
    pub fn set_score_info(&mut self, v: ScoreInfo) {
        self.score_info = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_score_info(&mut self) -> &mut ScoreInfo {
        if self.score_info.is_none() {
            self.score_info.set_default();
        }
        self.score_info.as_mut().unwrap()
    }

    // Take field
    pub fn take_score_info(&mut self) -> ScoreInfo {
        self.score_info.take().unwrap_or_else(|| ScoreInfo::new())
    }

    pub fn get_score_info(&self) -> &ScoreInfo {
        self.score_info.as_ref().unwrap_or_else(|| ScoreInfo::default_instance())
    }

    fn get_score_info_for_reflect(&self) -> &::protobuf::SingularPtrField<ScoreInfo> {
        &self.score_info
    }

    fn mut_score_info_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<ScoreInfo> {
        &mut self.score_info
    }

    // bool is_demolished = 6;

    pub fn clear_is_demolished(&mut self) {
        self.is_demolished = false;
    }

    // Param is passed by value, moved
    pub fn set_is_demolished(&mut self, v: bool) {
        self.is_demolished = v;
    }

    pub fn get_is_demolished(&self) -> bool {
        self.is_demolished
    }

    fn get_is_demolished_for_reflect(&self) -> &bool {
        &self.is_demolished
    }

    fn mut_is_demolished_for_reflect(&mut self) -> &mut bool {
        &mut self.is_demolished
    }

    // bool is_midair = 7;

    pub fn clear_is_midair(&mut self) {
        self.is_midair = false;
    }

    // Param is passed by value, moved
    pub fn set_is_midair(&mut self, v: bool) {
        self.is_midair = v;
    }

    pub fn get_is_midair(&self) -> bool {
        self.is_midair
    }

    fn get_is_midair_for_reflect(&self) -> &bool {
        &self.is_midair
    }

    fn mut_is_midair_for_reflect(&mut self) -> &mut bool {
        &mut self.is_midair
    }

    // bool is_supersonic = 8;

    pub fn clear_is_supersonic(&mut self) {
        self.is_supersonic = false;
    }

    // Param is passed by value, moved
    pub fn set_is_supersonic(&mut self, v: bool) {
        self.is_supersonic = v;
    }

    pub fn get_is_supersonic(&self) -> bool {
        self.is_supersonic
    }

    fn get_is_supersonic_for_reflect(&self) -> &bool {
        &self.is_supersonic
    }

    fn mut_is_supersonic_for_reflect(&mut self) -> &mut bool {
        &mut self.is_supersonic
    }

    // bool is_bot = 9;

    pub fn clear_is_bot(&mut self) {
        self.is_bot = false;
    }

    // Param is passed by value, moved
    pub fn set_is_bot(&mut self, v: bool) {
        self.is_bot = v;
    }

    pub fn get_is_bot(&self) -> bool {
        self.is_bot
    }

    fn get_is_bot_for_reflect(&self) -> &bool {
        &self.is_bot
    }

    fn mut_is_bot_for_reflect(&mut self) -> &mut bool {
        &mut self.is_bot
    }

    // bool jumped = 10;

    pub fn clear_jumped(&mut self) {
        self.jumped = false;
    }

    // Param is passed by value, moved
    pub fn set_jumped(&mut self, v: bool) {
        self.jumped = v;
    }

    pub fn get_jumped(&self) -> bool {
        self.jumped
    }

    fn get_jumped_for_reflect(&self) -> &bool {
        &self.jumped
    }

    fn mut_jumped_for_reflect(&mut self) -> &mut bool {
        &mut self.jumped
    }

    // bool double_jumped = 11;

    pub fn clear_double_jumped(&mut self) {
        self.double_jumped = false;
    }

    // Param is passed by value, moved
    pub fn set_double_jumped(&mut self, v: bool) {
        self.double_jumped = v;
    }

    pub fn get_double_jumped(&self) -> bool {
        self.double_jumped
    }

    fn get_double_jumped_for_reflect(&self) -> &bool {
        &self.double_jumped
    }

    fn mut_double_jumped_for_reflect(&mut self) -> &mut bool {
        &mut self.double_jumped
    }

    // string name = 12;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_name_for_reflect(&self) -> &::std::string::String {
        &self.name
    }

    fn mut_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // int32 team = 13;

    pub fn clear_team(&mut self) {
        self.team = 0;
    }

    // Param is passed by value, moved
    pub fn set_team(&mut self, v: i32) {
        self.team = v;
    }

    pub fn get_team(&self) -> i32 {
        self.team
    }

    fn get_team_for_reflect(&self) -> &i32 {
        &self.team
    }

    fn mut_team_for_reflect(&mut self) -> &mut i32 {
        &mut self.team
    }

    // int32 boost = 14;

    pub fn clear_boost(&mut self) {
        self.boost = 0;
    }

    // Param is passed by value, moved
    pub fn set_boost(&mut self, v: i32) {
        self.boost = v;
    }

    pub fn get_boost(&self) -> i32 {
        self.boost
    }

    fn get_boost_for_reflect(&self) -> &i32 {
        &self.boost
    }

    fn mut_boost_for_reflect(&mut self) -> &mut i32 {
        &mut self.boost
    }
}

impl ::protobuf::Message for PlayerInfo {
    fn is_initialized(&self) -> bool {
        for v in &self.location {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.rotation {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.velocity {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.angular_velocity {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.score_info {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.location)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.rotation)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.velocity)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.angular_velocity)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.score_info)?;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_demolished = tmp;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_midair = tmp;
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_supersonic = tmp;
                },
                9 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_bot = tmp;
                },
                10 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.jumped = tmp;
                },
                11 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.double_jumped = tmp;
                },
                12 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                13 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.team = tmp;
                },
                14 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.boost = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.location.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.rotation.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.velocity.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.angular_velocity.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.score_info.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if self.is_demolished != false {
            my_size += 2;
        }
        if self.is_midair != false {
            my_size += 2;
        }
        if self.is_supersonic != false {
            my_size += 2;
        }
        if self.is_bot != false {
            my_size += 2;
        }
        if self.jumped != false {
            my_size += 2;
        }
        if self.double_jumped != false {
            my_size += 2;
        }
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(12, &self.name);
        }
        if self.team != 0 {
            my_size += ::protobuf::rt::value_size(13, self.team, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.boost != 0 {
            my_size += ::protobuf::rt::value_size(14, self.boost, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.location.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.rotation.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.velocity.as_ref() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.angular_velocity.as_ref() {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.score_info.as_ref() {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if self.is_demolished != false {
            os.write_bool(6, self.is_demolished)?;
        }
        if self.is_midair != false {
            os.write_bool(7, self.is_midair)?;
        }
        if self.is_supersonic != false {
            os.write_bool(8, self.is_supersonic)?;
        }
        if self.is_bot != false {
            os.write_bool(9, self.is_bot)?;
        }
        if self.jumped != false {
            os.write_bool(10, self.jumped)?;
        }
        if self.double_jumped != false {
            os.write_bool(11, self.double_jumped)?;
        }
        if !self.name.is_empty() {
            os.write_string(12, &self.name)?;
        }
        if self.team != 0 {
            os.write_int32(13, self.team)?;
        }
        if self.boost != 0 {
            os.write_int32(14, self.boost)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for PlayerInfo {
    fn new() -> PlayerInfo {
        PlayerInfo::new()
    }

    fn descriptor_static(_: ::std::option::Option<PlayerInfo>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "location",
                    PlayerInfo::get_location_for_reflect,
                    PlayerInfo::mut_location_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Rotator>>(
                    "rotation",
                    PlayerInfo::get_rotation_for_reflect,
                    PlayerInfo::mut_rotation_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "velocity",
                    PlayerInfo::get_velocity_for_reflect,
                    PlayerInfo::mut_velocity_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "angular_velocity",
                    PlayerInfo::get_angular_velocity_for_reflect,
                    PlayerInfo::mut_angular_velocity_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<ScoreInfo>>(
                    "score_info",
                    PlayerInfo::get_score_info_for_reflect,
                    PlayerInfo::mut_score_info_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_demolished",
                    PlayerInfo::get_is_demolished_for_reflect,
                    PlayerInfo::mut_is_demolished_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_midair",
                    PlayerInfo::get_is_midair_for_reflect,
                    PlayerInfo::mut_is_midair_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_supersonic",
                    PlayerInfo::get_is_supersonic_for_reflect,
                    PlayerInfo::mut_is_supersonic_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_bot",
                    PlayerInfo::get_is_bot_for_reflect,
                    PlayerInfo::mut_is_bot_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "jumped",
                    PlayerInfo::get_jumped_for_reflect,
                    PlayerInfo::mut_jumped_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "double_jumped",
                    PlayerInfo::get_double_jumped_for_reflect,
                    PlayerInfo::mut_double_jumped_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    PlayerInfo::get_name_for_reflect,
                    PlayerInfo::mut_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "team",
                    PlayerInfo::get_team_for_reflect,
                    PlayerInfo::mut_team_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "boost",
                    PlayerInfo::get_boost_for_reflect,
                    PlayerInfo::mut_boost_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<PlayerInfo>(
                    "PlayerInfo",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for PlayerInfo {
    fn clear(&mut self) {
        self.clear_location();
        self.clear_rotation();
        self.clear_velocity();
        self.clear_angular_velocity();
        self.clear_score_info();
        self.clear_is_demolished();
        self.clear_is_midair();
        self.clear_is_supersonic();
        self.clear_is_bot();
        self.clear_jumped();
        self.clear_double_jumped();
        self.clear_name();
        self.clear_team();
        self.clear_boost();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PlayerInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PlayerInfo {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct BallInfo {
    // message fields
    pub location: ::protobuf::SingularPtrField<Vector3>,
    pub rotation: ::protobuf::SingularPtrField<Rotator>,
    pub velocity: ::protobuf::SingularPtrField<Vector3>,
    pub angular_velocity: ::protobuf::SingularPtrField<Vector3>,
    pub acceleration: ::protobuf::SingularPtrField<Vector3>,
    pub latest_touch: ::protobuf::SingularPtrField<Touch>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for BallInfo {}

impl BallInfo {
    pub fn new() -> BallInfo {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static BallInfo {
        static mut instance: ::protobuf::lazy::Lazy<BallInfo> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const BallInfo,
        };
        unsafe {
            instance.get(BallInfo::new)
        }
    }

    // .rlbot.api.Vector3 location = 1;

    pub fn clear_location(&mut self) {
        self.location.clear();
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    // Param is passed by value, moved
    pub fn set_location(&mut self, v: Vector3) {
        self.location = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_location(&mut self) -> &mut Vector3 {
        if self.location.is_none() {
            self.location.set_default();
        }
        self.location.as_mut().unwrap()
    }

    // Take field
    pub fn take_location(&mut self) -> Vector3 {
        self.location.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_location(&self) -> &Vector3 {
        self.location.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_location_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.location
    }

    fn mut_location_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.location
    }

    // .rlbot.api.Rotator rotation = 2;

    pub fn clear_rotation(&mut self) {
        self.rotation.clear();
    }

    pub fn has_rotation(&self) -> bool {
        self.rotation.is_some()
    }

    // Param is passed by value, moved
    pub fn set_rotation(&mut self, v: Rotator) {
        self.rotation = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_rotation(&mut self) -> &mut Rotator {
        if self.rotation.is_none() {
            self.rotation.set_default();
        }
        self.rotation.as_mut().unwrap()
    }

    // Take field
    pub fn take_rotation(&mut self) -> Rotator {
        self.rotation.take().unwrap_or_else(|| Rotator::new())
    }

    pub fn get_rotation(&self) -> &Rotator {
        self.rotation.as_ref().unwrap_or_else(|| Rotator::default_instance())
    }

    fn get_rotation_for_reflect(&self) -> &::protobuf::SingularPtrField<Rotator> {
        &self.rotation
    }

    fn mut_rotation_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Rotator> {
        &mut self.rotation
    }

    // .rlbot.api.Vector3 velocity = 3;

    pub fn clear_velocity(&mut self) {
        self.velocity.clear();
    }

    pub fn has_velocity(&self) -> bool {
        self.velocity.is_some()
    }

    // Param is passed by value, moved
    pub fn set_velocity(&mut self, v: Vector3) {
        self.velocity = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_velocity(&mut self) -> &mut Vector3 {
        if self.velocity.is_none() {
            self.velocity.set_default();
        }
        self.velocity.as_mut().unwrap()
    }

    // Take field
    pub fn take_velocity(&mut self) -> Vector3 {
        self.velocity.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_velocity(&self) -> &Vector3 {
        self.velocity.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_velocity_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.velocity
    }

    fn mut_velocity_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.velocity
    }

    // .rlbot.api.Vector3 angular_velocity = 4;

    pub fn clear_angular_velocity(&mut self) {
        self.angular_velocity.clear();
    }

    pub fn has_angular_velocity(&self) -> bool {
        self.angular_velocity.is_some()
    }

    // Param is passed by value, moved
    pub fn set_angular_velocity(&mut self, v: Vector3) {
        self.angular_velocity = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_angular_velocity(&mut self) -> &mut Vector3 {
        if self.angular_velocity.is_none() {
            self.angular_velocity.set_default();
        }
        self.angular_velocity.as_mut().unwrap()
    }

    // Take field
    pub fn take_angular_velocity(&mut self) -> Vector3 {
        self.angular_velocity.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_angular_velocity(&self) -> &Vector3 {
        self.angular_velocity.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_angular_velocity_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.angular_velocity
    }

    fn mut_angular_velocity_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.angular_velocity
    }

    // .rlbot.api.Vector3 acceleration = 5;

    pub fn clear_acceleration(&mut self) {
        self.acceleration.clear();
    }

    pub fn has_acceleration(&self) -> bool {
        self.acceleration.is_some()
    }

    // Param is passed by value, moved
    pub fn set_acceleration(&mut self, v: Vector3) {
        self.acceleration = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_acceleration(&mut self) -> &mut Vector3 {
        if self.acceleration.is_none() {
            self.acceleration.set_default();
        }
        self.acceleration.as_mut().unwrap()
    }

    // Take field
    pub fn take_acceleration(&mut self) -> Vector3 {
        self.acceleration.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_acceleration(&self) -> &Vector3 {
        self.acceleration.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_acceleration_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.acceleration
    }

    fn mut_acceleration_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.acceleration
    }

    // .rlbot.api.Touch latest_touch = 6;

    pub fn clear_latest_touch(&mut self) {
        self.latest_touch.clear();
    }

    pub fn has_latest_touch(&self) -> bool {
        self.latest_touch.is_some()
    }

    // Param is passed by value, moved
    pub fn set_latest_touch(&mut self, v: Touch) {
        self.latest_touch = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_latest_touch(&mut self) -> &mut Touch {
        if self.latest_touch.is_none() {
            self.latest_touch.set_default();
        }
        self.latest_touch.as_mut().unwrap()
    }

    // Take field
    pub fn take_latest_touch(&mut self) -> Touch {
        self.latest_touch.take().unwrap_or_else(|| Touch::new())
    }

    pub fn get_latest_touch(&self) -> &Touch {
        self.latest_touch.as_ref().unwrap_or_else(|| Touch::default_instance())
    }

    fn get_latest_touch_for_reflect(&self) -> &::protobuf::SingularPtrField<Touch> {
        &self.latest_touch
    }

    fn mut_latest_touch_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Touch> {
        &mut self.latest_touch
    }
}

impl ::protobuf::Message for BallInfo {
    fn is_initialized(&self) -> bool {
        for v in &self.location {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.rotation {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.velocity {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.angular_velocity {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.acceleration {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.latest_touch {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.location)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.rotation)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.velocity)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.angular_velocity)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.acceleration)?;
                },
                6 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.latest_touch)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.location.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.rotation.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.velocity.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.angular_velocity.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.acceleration.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.latest_touch.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.location.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.rotation.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.velocity.as_ref() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.angular_velocity.as_ref() {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.acceleration.as_ref() {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.latest_touch.as_ref() {
            os.write_tag(6, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for BallInfo {
    fn new() -> BallInfo {
        BallInfo::new()
    }

    fn descriptor_static(_: ::std::option::Option<BallInfo>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "location",
                    BallInfo::get_location_for_reflect,
                    BallInfo::mut_location_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Rotator>>(
                    "rotation",
                    BallInfo::get_rotation_for_reflect,
                    BallInfo::mut_rotation_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "velocity",
                    BallInfo::get_velocity_for_reflect,
                    BallInfo::mut_velocity_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "angular_velocity",
                    BallInfo::get_angular_velocity_for_reflect,
                    BallInfo::mut_angular_velocity_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "acceleration",
                    BallInfo::get_acceleration_for_reflect,
                    BallInfo::mut_acceleration_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Touch>>(
                    "latest_touch",
                    BallInfo::get_latest_touch_for_reflect,
                    BallInfo::mut_latest_touch_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<BallInfo>(
                    "BallInfo",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for BallInfo {
    fn clear(&mut self) {
        self.clear_location();
        self.clear_rotation();
        self.clear_velocity();
        self.clear_angular_velocity();
        self.clear_acceleration();
        self.clear_latest_touch();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for BallInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for BallInfo {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct BoostInfo {
    // message fields
    pub location: ::protobuf::SingularPtrField<Vector3>,
    pub is_active: bool,
    pub timer: i32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for BoostInfo {}

impl BoostInfo {
    pub fn new() -> BoostInfo {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static BoostInfo {
        static mut instance: ::protobuf::lazy::Lazy<BoostInfo> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const BoostInfo,
        };
        unsafe {
            instance.get(BoostInfo::new)
        }
    }

    // .rlbot.api.Vector3 location = 1;

    pub fn clear_location(&mut self) {
        self.location.clear();
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    // Param is passed by value, moved
    pub fn set_location(&mut self, v: Vector3) {
        self.location = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_location(&mut self) -> &mut Vector3 {
        if self.location.is_none() {
            self.location.set_default();
        }
        self.location.as_mut().unwrap()
    }

    // Take field
    pub fn take_location(&mut self) -> Vector3 {
        self.location.take().unwrap_or_else(|| Vector3::new())
    }

    pub fn get_location(&self) -> &Vector3 {
        self.location.as_ref().unwrap_or_else(|| Vector3::default_instance())
    }

    fn get_location_for_reflect(&self) -> &::protobuf::SingularPtrField<Vector3> {
        &self.location
    }

    fn mut_location_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Vector3> {
        &mut self.location
    }

    // bool is_active = 2;

    pub fn clear_is_active(&mut self) {
        self.is_active = false;
    }

    // Param is passed by value, moved
    pub fn set_is_active(&mut self, v: bool) {
        self.is_active = v;
    }

    pub fn get_is_active(&self) -> bool {
        self.is_active
    }

    fn get_is_active_for_reflect(&self) -> &bool {
        &self.is_active
    }

    fn mut_is_active_for_reflect(&mut self) -> &mut bool {
        &mut self.is_active
    }

    // int32 timer = 3;

    pub fn clear_timer(&mut self) {
        self.timer = 0;
    }

    // Param is passed by value, moved
    pub fn set_timer(&mut self, v: i32) {
        self.timer = v;
    }

    pub fn get_timer(&self) -> i32 {
        self.timer
    }

    fn get_timer_for_reflect(&self) -> &i32 {
        &self.timer
    }

    fn mut_timer_for_reflect(&mut self) -> &mut i32 {
        &mut self.timer
    }
}

impl ::protobuf::Message for BoostInfo {
    fn is_initialized(&self) -> bool {
        for v in &self.location {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.location)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_active = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.timer = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.location.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if self.is_active != false {
            my_size += 2;
        }
        if self.timer != 0 {
            my_size += ::protobuf::rt::value_size(3, self.timer, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.location.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if self.is_active != false {
            os.write_bool(2, self.is_active)?;
        }
        if self.timer != 0 {
            os.write_int32(3, self.timer)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for BoostInfo {
    fn new() -> BoostInfo {
        BoostInfo::new()
    }

    fn descriptor_static(_: ::std::option::Option<BoostInfo>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Vector3>>(
                    "location",
                    BoostInfo::get_location_for_reflect,
                    BoostInfo::mut_location_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_active",
                    BoostInfo::get_is_active_for_reflect,
                    BoostInfo::mut_is_active_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "timer",
                    BoostInfo::get_timer_for_reflect,
                    BoostInfo::mut_timer_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<BoostInfo>(
                    "BoostInfo",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for BoostInfo {
    fn clear(&mut self) {
        self.clear_location();
        self.clear_is_active();
        self.clear_timer();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for BoostInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for BoostInfo {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct GameInfo {
    // message fields
    pub seconds_elapsed: f32,
    pub game_time_remaining: f32,
    pub is_overtime: bool,
    pub is_unlimited_time: bool,
    pub is_round_active: bool,
    pub is_kickoff_pause: bool,
    pub is_match_ended: bool,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for GameInfo {}

impl GameInfo {
    pub fn new() -> GameInfo {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static GameInfo {
        static mut instance: ::protobuf::lazy::Lazy<GameInfo> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const GameInfo,
        };
        unsafe {
            instance.get(GameInfo::new)
        }
    }

    // float seconds_elapsed = 1;

    pub fn clear_seconds_elapsed(&mut self) {
        self.seconds_elapsed = 0.;
    }

    // Param is passed by value, moved
    pub fn set_seconds_elapsed(&mut self, v: f32) {
        self.seconds_elapsed = v;
    }

    pub fn get_seconds_elapsed(&self) -> f32 {
        self.seconds_elapsed
    }

    fn get_seconds_elapsed_for_reflect(&self) -> &f32 {
        &self.seconds_elapsed
    }

    fn mut_seconds_elapsed_for_reflect(&mut self) -> &mut f32 {
        &mut self.seconds_elapsed
    }

    // float game_time_remaining = 2;

    pub fn clear_game_time_remaining(&mut self) {
        self.game_time_remaining = 0.;
    }

    // Param is passed by value, moved
    pub fn set_game_time_remaining(&mut self, v: f32) {
        self.game_time_remaining = v;
    }

    pub fn get_game_time_remaining(&self) -> f32 {
        self.game_time_remaining
    }

    fn get_game_time_remaining_for_reflect(&self) -> &f32 {
        &self.game_time_remaining
    }

    fn mut_game_time_remaining_for_reflect(&mut self) -> &mut f32 {
        &mut self.game_time_remaining
    }

    // bool is_overtime = 3;

    pub fn clear_is_overtime(&mut self) {
        self.is_overtime = false;
    }

    // Param is passed by value, moved
    pub fn set_is_overtime(&mut self, v: bool) {
        self.is_overtime = v;
    }

    pub fn get_is_overtime(&self) -> bool {
        self.is_overtime
    }

    fn get_is_overtime_for_reflect(&self) -> &bool {
        &self.is_overtime
    }

    fn mut_is_overtime_for_reflect(&mut self) -> &mut bool {
        &mut self.is_overtime
    }

    // bool is_unlimited_time = 4;

    pub fn clear_is_unlimited_time(&mut self) {
        self.is_unlimited_time = false;
    }

    // Param is passed by value, moved
    pub fn set_is_unlimited_time(&mut self, v: bool) {
        self.is_unlimited_time = v;
    }

    pub fn get_is_unlimited_time(&self) -> bool {
        self.is_unlimited_time
    }

    fn get_is_unlimited_time_for_reflect(&self) -> &bool {
        &self.is_unlimited_time
    }

    fn mut_is_unlimited_time_for_reflect(&mut self) -> &mut bool {
        &mut self.is_unlimited_time
    }

    // bool is_round_active = 5;

    pub fn clear_is_round_active(&mut self) {
        self.is_round_active = false;
    }

    // Param is passed by value, moved
    pub fn set_is_round_active(&mut self, v: bool) {
        self.is_round_active = v;
    }

    pub fn get_is_round_active(&self) -> bool {
        self.is_round_active
    }

    fn get_is_round_active_for_reflect(&self) -> &bool {
        &self.is_round_active
    }

    fn mut_is_round_active_for_reflect(&mut self) -> &mut bool {
        &mut self.is_round_active
    }

    // bool is_kickoff_pause = 6;

    pub fn clear_is_kickoff_pause(&mut self) {
        self.is_kickoff_pause = false;
    }

    // Param is passed by value, moved
    pub fn set_is_kickoff_pause(&mut self, v: bool) {
        self.is_kickoff_pause = v;
    }

    pub fn get_is_kickoff_pause(&self) -> bool {
        self.is_kickoff_pause
    }

    fn get_is_kickoff_pause_for_reflect(&self) -> &bool {
        &self.is_kickoff_pause
    }

    fn mut_is_kickoff_pause_for_reflect(&mut self) -> &mut bool {
        &mut self.is_kickoff_pause
    }

    // bool is_match_ended = 7;

    pub fn clear_is_match_ended(&mut self) {
        self.is_match_ended = false;
    }

    // Param is passed by value, moved
    pub fn set_is_match_ended(&mut self, v: bool) {
        self.is_match_ended = v;
    }

    pub fn get_is_match_ended(&self) -> bool {
        self.is_match_ended
    }

    fn get_is_match_ended_for_reflect(&self) -> &bool {
        &self.is_match_ended
    }

    fn mut_is_match_ended_for_reflect(&mut self) -> &mut bool {
        &mut self.is_match_ended
    }
}

impl ::protobuf::Message for GameInfo {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.seconds_elapsed = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_float()?;
                    self.game_time_remaining = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_overtime = tmp;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_unlimited_time = tmp;
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_round_active = tmp;
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_kickoff_pause = tmp;
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_match_ended = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.seconds_elapsed != 0. {
            my_size += 5;
        }
        if self.game_time_remaining != 0. {
            my_size += 5;
        }
        if self.is_overtime != false {
            my_size += 2;
        }
        if self.is_unlimited_time != false {
            my_size += 2;
        }
        if self.is_round_active != false {
            my_size += 2;
        }
        if self.is_kickoff_pause != false {
            my_size += 2;
        }
        if self.is_match_ended != false {
            my_size += 2;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.seconds_elapsed != 0. {
            os.write_float(1, self.seconds_elapsed)?;
        }
        if self.game_time_remaining != 0. {
            os.write_float(2, self.game_time_remaining)?;
        }
        if self.is_overtime != false {
            os.write_bool(3, self.is_overtime)?;
        }
        if self.is_unlimited_time != false {
            os.write_bool(4, self.is_unlimited_time)?;
        }
        if self.is_round_active != false {
            os.write_bool(5, self.is_round_active)?;
        }
        if self.is_kickoff_pause != false {
            os.write_bool(6, self.is_kickoff_pause)?;
        }
        if self.is_match_ended != false {
            os.write_bool(7, self.is_match_ended)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for GameInfo {
    fn new() -> GameInfo {
        GameInfo::new()
    }

    fn descriptor_static(_: ::std::option::Option<GameInfo>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "seconds_elapsed",
                    GameInfo::get_seconds_elapsed_for_reflect,
                    GameInfo::mut_seconds_elapsed_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeFloat>(
                    "game_time_remaining",
                    GameInfo::get_game_time_remaining_for_reflect,
                    GameInfo::mut_game_time_remaining_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_overtime",
                    GameInfo::get_is_overtime_for_reflect,
                    GameInfo::mut_is_overtime_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_unlimited_time",
                    GameInfo::get_is_unlimited_time_for_reflect,
                    GameInfo::mut_is_unlimited_time_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_round_active",
                    GameInfo::get_is_round_active_for_reflect,
                    GameInfo::mut_is_round_active_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_kickoff_pause",
                    GameInfo::get_is_kickoff_pause_for_reflect,
                    GameInfo::mut_is_kickoff_pause_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_match_ended",
                    GameInfo::get_is_match_ended_for_reflect,
                    GameInfo::mut_is_match_ended_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<GameInfo>(
                    "GameInfo",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for GameInfo {
    fn clear(&mut self) {
        self.clear_seconds_elapsed();
        self.clear_game_time_remaining();
        self.clear_is_overtime();
        self.clear_is_unlimited_time();
        self.clear_is_round_active();
        self.clear_is_kickoff_pause();
        self.clear_is_match_ended();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for GameInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for GameInfo {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct GameTickPacket {
    // message fields
    pub players: ::protobuf::RepeatedField<PlayerInfo>,
    pub player_index: i32,
    pub boost_pads: ::protobuf::RepeatedField<BoostInfo>,
    pub ball: ::protobuf::SingularPtrField<BallInfo>,
    pub game_info: ::protobuf::SingularPtrField<GameInfo>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for GameTickPacket {}

impl GameTickPacket {
    pub fn new() -> GameTickPacket {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static GameTickPacket {
        static mut instance: ::protobuf::lazy::Lazy<GameTickPacket> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const GameTickPacket,
        };
        unsafe {
            instance.get(GameTickPacket::new)
        }
    }

    // repeated .rlbot.api.PlayerInfo players = 1;

    pub fn clear_players(&mut self) {
        self.players.clear();
    }

    // Param is passed by value, moved
    pub fn set_players(&mut self, v: ::protobuf::RepeatedField<PlayerInfo>) {
        self.players = v;
    }

    // Mutable pointer to the field.
    pub fn mut_players(&mut self) -> &mut ::protobuf::RepeatedField<PlayerInfo> {
        &mut self.players
    }

    // Take field
    pub fn take_players(&mut self) -> ::protobuf::RepeatedField<PlayerInfo> {
        ::std::mem::replace(&mut self.players, ::protobuf::RepeatedField::new())
    }

    pub fn get_players(&self) -> &[PlayerInfo] {
        &self.players
    }

    fn get_players_for_reflect(&self) -> &::protobuf::RepeatedField<PlayerInfo> {
        &self.players
    }

    fn mut_players_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<PlayerInfo> {
        &mut self.players
    }

    // int32 player_index = 2;

    pub fn clear_player_index(&mut self) {
        self.player_index = 0;
    }

    // Param is passed by value, moved
    pub fn set_player_index(&mut self, v: i32) {
        self.player_index = v;
    }

    pub fn get_player_index(&self) -> i32 {
        self.player_index
    }

    fn get_player_index_for_reflect(&self) -> &i32 {
        &self.player_index
    }

    fn mut_player_index_for_reflect(&mut self) -> &mut i32 {
        &mut self.player_index
    }

    // repeated .rlbot.api.BoostInfo boost_pads = 3;

    pub fn clear_boost_pads(&mut self) {
        self.boost_pads.clear();
    }

    // Param is passed by value, moved
    pub fn set_boost_pads(&mut self, v: ::protobuf::RepeatedField<BoostInfo>) {
        self.boost_pads = v;
    }

    // Mutable pointer to the field.
    pub fn mut_boost_pads(&mut self) -> &mut ::protobuf::RepeatedField<BoostInfo> {
        &mut self.boost_pads
    }

    // Take field
    pub fn take_boost_pads(&mut self) -> ::protobuf::RepeatedField<BoostInfo> {
        ::std::mem::replace(&mut self.boost_pads, ::protobuf::RepeatedField::new())
    }

    pub fn get_boost_pads(&self) -> &[BoostInfo] {
        &self.boost_pads
    }

    fn get_boost_pads_for_reflect(&self) -> &::protobuf::RepeatedField<BoostInfo> {
        &self.boost_pads
    }

    fn mut_boost_pads_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<BoostInfo> {
        &mut self.boost_pads
    }

    // .rlbot.api.BallInfo ball = 4;

    pub fn clear_ball(&mut self) {
        self.ball.clear();
    }

    pub fn has_ball(&self) -> bool {
        self.ball.is_some()
    }

    // Param is passed by value, moved
    pub fn set_ball(&mut self, v: BallInfo) {
        self.ball = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_ball(&mut self) -> &mut BallInfo {
        if self.ball.is_none() {
            self.ball.set_default();
        }
        self.ball.as_mut().unwrap()
    }

    // Take field
    pub fn take_ball(&mut self) -> BallInfo {
        self.ball.take().unwrap_or_else(|| BallInfo::new())
    }

    pub fn get_ball(&self) -> &BallInfo {
        self.ball.as_ref().unwrap_or_else(|| BallInfo::default_instance())
    }

    fn get_ball_for_reflect(&self) -> &::protobuf::SingularPtrField<BallInfo> {
        &self.ball
    }

    fn mut_ball_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<BallInfo> {
        &mut self.ball
    }

    // .rlbot.api.GameInfo game_info = 5;

    pub fn clear_game_info(&mut self) {
        self.game_info.clear();
    }

    pub fn has_game_info(&self) -> bool {
        self.game_info.is_some()
    }

    // Param is passed by value, moved
    pub fn set_game_info(&mut self, v: GameInfo) {
        self.game_info = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_game_info(&mut self) -> &mut GameInfo {
        if self.game_info.is_none() {
            self.game_info.set_default();
        }
        self.game_info.as_mut().unwrap()
    }

    // Take field
    pub fn take_game_info(&mut self) -> GameInfo {
        self.game_info.take().unwrap_or_else(|| GameInfo::new())
    }

    pub fn get_game_info(&self) -> &GameInfo {
        self.game_info.as_ref().unwrap_or_else(|| GameInfo::default_instance())
    }

    fn get_game_info_for_reflect(&self) -> &::protobuf::SingularPtrField<GameInfo> {
        &self.game_info
    }

    fn mut_game_info_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<GameInfo> {
        &mut self.game_info
    }
}

impl ::protobuf::Message for GameTickPacket {
    fn is_initialized(&self) -> bool {
        for v in &self.players {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.boost_pads {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.ball {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.game_info {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.players)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.player_index = tmp;
                },
                3 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.boost_pads)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.ball)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.game_info)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.players {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        if self.player_index != 0 {
            my_size += ::protobuf::rt::value_size(2, self.player_index, ::protobuf::wire_format::WireTypeVarint);
        }
        for value in &self.boost_pads {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        if let Some(ref v) = self.ball.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.game_info.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.players {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        if self.player_index != 0 {
            os.write_int32(2, self.player_index)?;
        }
        for v in &self.boost_pads {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        if let Some(ref v) = self.ball.as_ref() {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.game_info.as_ref() {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for GameTickPacket {
    fn new() -> GameTickPacket {
        GameTickPacket::new()
    }

    fn descriptor_static(_: ::std::option::Option<GameTickPacket>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<PlayerInfo>>(
                    "players",
                    GameTickPacket::get_players_for_reflect,
                    GameTickPacket::mut_players_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "player_index",
                    GameTickPacket::get_player_index_for_reflect,
                    GameTickPacket::mut_player_index_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<BoostInfo>>(
                    "boost_pads",
                    GameTickPacket::get_boost_pads_for_reflect,
                    GameTickPacket::mut_boost_pads_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<BallInfo>>(
                    "ball",
                    GameTickPacket::get_ball_for_reflect,
                    GameTickPacket::mut_ball_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<GameInfo>>(
                    "game_info",
                    GameTickPacket::get_game_info_for_reflect,
                    GameTickPacket::mut_game_info_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<GameTickPacket>(
                    "GameTickPacket",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for GameTickPacket {
    fn clear(&mut self) {
        self.clear_players();
        self.clear_player_index();
        self.clear_boost_pads();
        self.clear_ball();
        self.clear_game_info();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for GameTickPacket {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for GameTickPacket {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0fgame_data.proto\x12\trlbot.api\"\xc7\x01\n\x0fControllerState\x12\
    \x1a\n\x08throttle\x18\x01\x20\x01(\x02R\x08throttle\x12\x14\n\x05steer\
    \x18\x02\x20\x01(\x02R\x05steer\x12\x14\n\x05pitch\x18\x03\x20\x01(\x02R\
    \x05pitch\x12\x10\n\x03yaw\x18\x04\x20\x01(\x02R\x03yaw\x12\x12\n\x04rol\
    l\x18\x05\x20\x01(\x02R\x04roll\x12\x12\n\x04jump\x18\x06\x20\x01(\x08R\
    \x04jump\x12\x14\n\x05boost\x18\x07\x20\x01(\x08R\x05boost\x12\x1c\n\tha\
    ndbrake\x18\x08\x20\x01(\x08R\thandbrake\"3\n\x07Vector3\x12\x0c\n\x01x\
    \x18\x01\x20\x01(\x02R\x01x\x12\x0c\n\x01y\x18\x02\x20\x01(\x02R\x01y\
    \x12\x0c\n\x01z\x18\x03\x20\x01(\x02R\x01z\"E\n\x07Rotator\x12\x14\n\x05\
    pitch\x18\x01\x20\x01(\x02R\x05pitch\x12\x10\n\x03yaw\x18\x02\x20\x01(\
    \x02R\x03yaw\x12\x12\n\x04roll\x18\x03\x20\x01(\x02R\x04roll\"\xa7\x01\n\
    \x05Touch\x12\x1f\n\x0bplayer_name\x18\x01\x20\x01(\tR\nplayerName\x12!\
    \n\x0cgame_seconds\x18\x02\x20\x01(\x02R\x0bgameSeconds\x12.\n\x08locati\
    on\x18\x03\x20\x01(\x0b2\x12.rlbot.api.Vector3R\x08location\x12*\n\x06no\
    rmal\x18\x04\x20\x01(\x0b2\x12.rlbot.api.Vector3R\x06normal\"\xbc\x01\n\
    \tScoreInfo\x12\x14\n\x05score\x18\x01\x20\x01(\x05R\x05score\x12\x14\n\
    \x05goals\x18\x02\x20\x01(\x05R\x05goals\x12\x1b\n\town_goals\x18\x03\
    \x20\x01(\x05R\x08ownGoals\x12\x18\n\x07assists\x18\x04\x20\x01(\x05R\
    \x07assists\x12\x14\n\x05saves\x18\x05\x20\x01(\x05R\x05saves\x12\x14\n\
    \x05shots\x18\x06\x20\x01(\x05R\x05shots\x12\x20\n\x0bdemolitions\x18\
    \x07\x20\x01(\x05R\x0bdemolitions\"\x89\x04\n\nPlayerInfo\x12.\n\x08loca\
    tion\x18\x01\x20\x01(\x0b2\x12.rlbot.api.Vector3R\x08location\x12.\n\x08\
    rotation\x18\x02\x20\x01(\x0b2\x12.rlbot.api.RotatorR\x08rotation\x12.\n\
    \x08velocity\x18\x03\x20\x01(\x0b2\x12.rlbot.api.Vector3R\x08velocity\
    \x12=\n\x10angular_velocity\x18\x04\x20\x01(\x0b2\x12.rlbot.api.Vector3R\
    \x0fangularVelocity\x123\n\nscore_info\x18\x05\x20\x01(\x0b2\x14.rlbot.a\
    pi.ScoreInfoR\tscoreInfo\x12#\n\ris_demolished\x18\x06\x20\x01(\x08R\x0c\
    isDemolished\x12\x1b\n\tis_midair\x18\x07\x20\x01(\x08R\x08isMidair\x12#\
    \n\ris_supersonic\x18\x08\x20\x01(\x08R\x0cisSupersonic\x12\x15\n\x06is_\
    bot\x18\t\x20\x01(\x08R\x05isBot\x12\x16\n\x06jumped\x18\n\x20\x01(\x08R\
    \x06jumped\x12#\n\rdouble_jumped\x18\x0b\x20\x01(\x08R\x0cdoubleJumped\
    \x12\x12\n\x04name\x18\x0c\x20\x01(\tR\x04name\x12\x12\n\x04team\x18\r\
    \x20\x01(\x05R\x04team\x12\x14\n\x05boost\x18\x0e\x20\x01(\x05R\x05boost\
    \"\xc6\x02\n\x08BallInfo\x12.\n\x08location\x18\x01\x20\x01(\x0b2\x12.rl\
    bot.api.Vector3R\x08location\x12.\n\x08rotation\x18\x02\x20\x01(\x0b2\
    \x12.rlbot.api.RotatorR\x08rotation\x12.\n\x08velocity\x18\x03\x20\x01(\
    \x0b2\x12.rlbot.api.Vector3R\x08velocity\x12=\n\x10angular_velocity\x18\
    \x04\x20\x01(\x0b2\x12.rlbot.api.Vector3R\x0fangularVelocity\x126\n\x0ca\
    cceleration\x18\x05\x20\x01(\x0b2\x12.rlbot.api.Vector3R\x0cacceleration\
    \x123\n\x0clatest_touch\x18\x06\x20\x01(\x0b2\x10.rlbot.api.TouchR\x0bla\
    testTouch\"n\n\tBoostInfo\x12.\n\x08location\x18\x01\x20\x01(\x0b2\x12.r\
    lbot.api.Vector3R\x08location\x12\x1b\n\tis_active\x18\x02\x20\x01(\x08R\
    \x08isActive\x12\x14\n\x05timer\x18\x03\x20\x01(\x05R\x05timer\"\xa8\x02\
    \n\x08GameInfo\x12'\n\x0fseconds_elapsed\x18\x01\x20\x01(\x02R\x0esecond\
    sElapsed\x12.\n\x13game_time_remaining\x18\x02\x20\x01(\x02R\x11gameTime\
    Remaining\x12\x1f\n\x0bis_overtime\x18\x03\x20\x01(\x08R\nisOvertime\x12\
    *\n\x11is_unlimited_time\x18\x04\x20\x01(\x08R\x0fisUnlimitedTime\x12&\n\
    \x0fis_round_active\x18\x05\x20\x01(\x08R\risRoundActive\x12(\n\x10is_ki\
    ckoff_pause\x18\x06\x20\x01(\x08R\x0eisKickoffPause\x12$\n\x0eis_match_e\
    nded\x18\x07\x20\x01(\x08R\x0cisMatchEnded\"\xf4\x01\n\x0eGameTickPacket\
    \x12/\n\x07players\x18\x01\x20\x03(\x0b2\x15.rlbot.api.PlayerInfoR\x07pl\
    ayers\x12!\n\x0cplayer_index\x18\x02\x20\x01(\x05R\x0bplayerIndex\x123\n\
    \nboost_pads\x18\x03\x20\x03(\x0b2\x14.rlbot.api.BoostInfoR\tboostPads\
    \x12'\n\x04ball\x18\x04\x20\x01(\x0b2\x13.rlbot.api.BallInfoR\x04ball\
    \x120\n\tgame_info\x18\x05\x20\x01(\x0b2\x13.rlbot.api.GameInfoR\x08game\
    Info2T\n\x03Bot\x12M\n\x12GetControllerState\x12\x19.rlbot.api.GameTickP\
    acket\x1a\x1a.rlbot.api.ControllerState\"\0b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
