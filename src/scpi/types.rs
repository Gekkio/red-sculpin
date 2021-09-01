// SPDX-FileCopyrightText: 2020-2021 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::convert::TryFrom;

use crate::{
    decode::Decoder,
    encode::{EncodeSink, Encoder},
    program_data::ProgramData,
    response_data::ResponseData,
    ByteSource,
};

/// Special program data that allows the instrument to select a numeric value.
///
/// Reference: SCPI 1999.0: 7.2.1.1 - DEFault
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DefaultValue;

impl ProgramData for DefaultValue {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        // Reference: `SCPI 1999.0: 7.2.1.1 - DEFault`
        encoder.encode_characters("DEF")
    }
}

/// Special program data that refers to a numeric limit value.
///
/// Reference: SCPI 1999.0: 7.2.1.2 - MINimum|MAXimum
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Limit {
    Min,
    Max,
}

impl ProgramData for Limit {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        // Reference: `SCPI 1999.0: 7.2.1.2 - MINimum|MAXimum`
        encoder.encode_characters(match self {
            Limit::Min => "MIN",
            Limit::Max => "MAX",
        })
    }
}

/// Special program data that refers to a numeric step direction.
///
/// Reference: SCPI 1999.0: 7.2.1.3 - UP|DOWN
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
}

impl ProgramData for Direction {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        encoder.begin_program_data()?;
        // Reference: `SCPI 1999.0: 7.2.1.3 - UP|DOWN`
        encoder.encode_characters(match self {
            Direction::Up => "UP",
            Direction::Down => "DOWN",
        })
    }
}

/// Standard error/event code defined by SCPI 1999.0
///
/// Reference: SCPI 1999.0: 21.8 - :ERRor Subsystem
#[repr(i16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StandardErrorCode {
    // Reference: SCPI 1999.0: 21.8.9 - Command Error
    CommandError = -100,
    InvalidCharacter = -101,
    SyntaxError = -102,
    InvalidSeparator = -103,
    DataTypeError = -104,
    GetNotAllowed = -105,
    ParameterNotAllowed = -108,
    MissingParameter = -109,
    CommandHeaderError = -110,
    HeaderSeparatorError = -111,
    ProgramMnemonicTooLong = -112,
    UndefinedHeader = -113,
    HeaderSuffixOutOfRange = -114,
    UnexpectedNumberOfParameters = -115,
    NumericDataError = -120,
    InvalidCharacterInNumber = -121,
    ExponentTooLarge = -123,
    TooManyDigits = -124,
    NumericDataNotAllowed = -128,
    SuffixError = -130,
    InvalidSuffix = -131,
    SuffixTooLong = -134,
    SuffixNotAllowed = -138,
    CharacterDataError = -140,
    InvalidCharacterData = -141,
    CharacterDataTooLong = -144,
    CharacterDataNotAllowed = -148,
    StringDataError = -150,
    InvalidStringData = -151,
    StringDataNotAllowed = -158,
    BlockDataError = -160,
    InvalidBlockData = -161,
    BlockDataNotAllowed = -168,
    ExpressionError = -170,
    InvalidExpression = -171,
    ExpressionDataNotAllowed = -178,
    MacroCommandError = -180,
    InvalidOutsideMacroDefinition = -181,
    InvalidInsideMacroDefinition = -183,
    MacroParameterCommandError = -184,
    // Reference: SCPI 1999.0: 21.8.10 - Execution Error
    ExecutionError = -200,
    InvalidWhileInLocal = -201,
    SettingsLostDueToRtl = -202,
    CommandProtected = -203,
    TriggerError = -210,
    TriggerIgnored = -211,
    ArmIgnored = -212,
    InitIgnored = -213,
    TriggerDeadlock = -214,
    ArmDeadlock = -215,
    ParameterError = -220,
    SettingsConflict = -221,
    DataOutOfRange = -222,
    TooMuchData = -223,
    IllegalParameterValue = -224,
    OutOfMemoryForOperation = -225,
    ListNotSameLength = -226,
    DataCorruptOrStale = -230,
    DataQuestionable = -231,
    InvalidFormat = -232,
    InvalidVersion = -233,
    HardwareError = -240,
    HardwareMissing = -241,
    MassStorageError = -250,
    MissingMassStorage = -251,
    MissingMedia = -252,
    CorruptMedia = -253,
    MediaFull = -254,
    DirectoryFull = -255,
    FileNameNotFound = -256,
    FileNameError = -257,
    MediaProtected = -258,
    ExpressionExecutionError = -260,
    MathErrorInExpression = -261,
    MacroError = -270,
    MacroSyntaxError = -271,
    MacroExecutionError = -272,
    IllegalMacroLabel = -273,
    MacroParameterError = -274,
    MacroDefinitionTooLong = -275,
    MacroRecursionError = -276,
    MacroRedefinitionNotAllowed = -277,
    MacroHeaderNotFound = -278,
    ProgramError = -280,
    CannotCreateProgram = -281,
    IllegalProgramName = -282,
    IllegalVariableName = -283,
    ProgramCurrentlyRunning = -284,
    ProgramSyntaxError = -285,
    ProgramRuntimeError = -286,
    MemoryUseError = -290,
    OutOfMemory = -291,
    ReferencedNameDoesNotExist = -292,
    ReferencedNameAlreadyExists = -293,
    IncompatibleType = -294,
    // Reference: SCPI 1999.0: 21.8.11 - Device-Specific Error
    DeviceSpecificError = -300,
    SystemError = -310,
    MemoryError = -311,
    PudMemoryLost = -312,
    CalibrationMemoryLost = -313,
    SaveRecallMemoryLost = -314,
    ConfigurationMemoryLost = -315,
    StorageFault = -320,
    DeviceOutOfMemory = -321,
    SelfTestFailed = -330,
    CalibrationFailed = -340,
    QueueOverflow = -350,
    CommunicationError = -360,
    ParityErrorInProgramMessage = -361,
    FramingErrorInProgramMessage = -362,
    InputBufferOverrun = -363,
    TimeOutError = -365,
    // Reference: SCPI 1999.0: 21.8.12 - Query Error
    QueryError = -400,
    QueryInterrupted = -410,
    QueryUnterminated = -420,
    QueryDeadlocked = -430,
    QueryUnterminatedAfterIndefiniteResponse = -440,
    // Reference: SCPI 1999.0: 21.8.13 - Power On Event
    PowerOn = -500,
    // Reference: SCPI 1999.0: 21.8.14 - User Request Event
    UserRequest = -600,
    // Reference: SCPI 1999.0: 21.8.15 - Request Control Event
    RequestControl = -700,
    // Reference: SCPI 1999.0: 21.8.16 - Operation Complete Event
    OperationComplete = -800,
}

impl TryFrom<i16> for StandardErrorCode {
    type Error = i16;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        use StandardErrorCode::*;
        match value {
            -100 => Ok(CommandError),
            -101 => Ok(InvalidCharacter),
            -102 => Ok(SyntaxError),
            -103 => Ok(InvalidSeparator),
            -104 => Ok(DataTypeError),
            -105 => Ok(GetNotAllowed),
            -108 => Ok(ParameterNotAllowed),
            -109 => Ok(MissingParameter),
            -110 => Ok(CommandHeaderError),
            -111 => Ok(HeaderSeparatorError),
            -112 => Ok(ProgramMnemonicTooLong),
            -113 => Ok(UndefinedHeader),
            -114 => Ok(HeaderSuffixOutOfRange),
            -115 => Ok(UnexpectedNumberOfParameters),
            -120 => Ok(NumericDataError),
            -121 => Ok(InvalidCharacterInNumber),
            -123 => Ok(ExponentTooLarge),
            -124 => Ok(TooManyDigits),
            -128 => Ok(NumericDataNotAllowed),
            -130 => Ok(SuffixError),
            -131 => Ok(InvalidSuffix),
            -134 => Ok(SuffixTooLong),
            -138 => Ok(SuffixNotAllowed),
            -140 => Ok(CharacterDataError),
            -141 => Ok(InvalidCharacterData),
            -144 => Ok(CharacterDataTooLong),
            -148 => Ok(CharacterDataNotAllowed),
            -150 => Ok(StringDataError),
            -151 => Ok(InvalidStringData),
            -158 => Ok(StringDataNotAllowed),
            -160 => Ok(BlockDataError),
            -161 => Ok(InvalidBlockData),
            -168 => Ok(BlockDataNotAllowed),
            -170 => Ok(ExpressionError),
            -171 => Ok(InvalidExpression),
            -178 => Ok(ExpressionDataNotAllowed),
            -180 => Ok(MacroCommandError),
            -181 => Ok(InvalidOutsideMacroDefinition),
            -183 => Ok(InvalidInsideMacroDefinition),
            -184 => Ok(MacroParameterCommandError),
            -200 => Ok(ExecutionError),
            -201 => Ok(InvalidWhileInLocal),
            -202 => Ok(SettingsLostDueToRtl),
            -203 => Ok(CommandProtected),
            -210 => Ok(TriggerError),
            -211 => Ok(TriggerIgnored),
            -212 => Ok(ArmIgnored),
            -213 => Ok(InitIgnored),
            -214 => Ok(TriggerDeadlock),
            -215 => Ok(ArmDeadlock),
            -220 => Ok(ParameterError),
            -221 => Ok(SettingsConflict),
            -222 => Ok(DataOutOfRange),
            -223 => Ok(TooMuchData),
            -224 => Ok(IllegalParameterValue),
            -225 => Ok(OutOfMemoryForOperation),
            -226 => Ok(ListNotSameLength),
            -230 => Ok(DataCorruptOrStale),
            -231 => Ok(DataQuestionable),
            -232 => Ok(InvalidFormat),
            -233 => Ok(InvalidVersion),
            -240 => Ok(HardwareError),
            -241 => Ok(HardwareMissing),
            -250 => Ok(MassStorageError),
            -251 => Ok(MissingMassStorage),
            -252 => Ok(MissingMedia),
            -253 => Ok(CorruptMedia),
            -254 => Ok(MediaFull),
            -255 => Ok(DirectoryFull),
            -256 => Ok(FileNameNotFound),
            -257 => Ok(FileNameError),
            -258 => Ok(MediaProtected),
            -260 => Ok(ExpressionExecutionError),
            -261 => Ok(MathErrorInExpression),
            -270 => Ok(MacroError),
            -271 => Ok(MacroSyntaxError),
            -272 => Ok(MacroExecutionError),
            -273 => Ok(IllegalMacroLabel),
            -274 => Ok(MacroParameterError),
            -275 => Ok(MacroDefinitionTooLong),
            -276 => Ok(MacroRecursionError),
            -277 => Ok(MacroRedefinitionNotAllowed),
            -278 => Ok(MacroHeaderNotFound),
            -280 => Ok(ProgramError),
            -281 => Ok(CannotCreateProgram),
            -282 => Ok(IllegalProgramName),
            -283 => Ok(IllegalVariableName),
            -284 => Ok(ProgramCurrentlyRunning),
            -285 => Ok(ProgramSyntaxError),
            -286 => Ok(ProgramRuntimeError),
            -290 => Ok(MemoryUseError),
            -291 => Ok(OutOfMemory),
            -292 => Ok(ReferencedNameDoesNotExist),
            -293 => Ok(ReferencedNameAlreadyExists),
            -294 => Ok(IncompatibleType),
            -300 => Ok(DeviceSpecificError),
            -310 => Ok(SystemError),
            -311 => Ok(MemoryError),
            -312 => Ok(PudMemoryLost),
            -313 => Ok(CalibrationMemoryLost),
            -314 => Ok(SaveRecallMemoryLost),
            -315 => Ok(ConfigurationMemoryLost),
            -320 => Ok(StorageFault),
            -321 => Ok(DeviceOutOfMemory),
            -330 => Ok(SelfTestFailed),
            -340 => Ok(CalibrationFailed),
            -350 => Ok(QueueOverflow),
            -360 => Ok(CommunicationError),
            -361 => Ok(ParityErrorInProgramMessage),
            -362 => Ok(FramingErrorInProgramMessage),
            -363 => Ok(InputBufferOverrun),
            -365 => Ok(TimeOutError),
            -400 => Ok(QueryError),
            -410 => Ok(QueryInterrupted),
            -420 => Ok(QueryUnterminated),
            -430 => Ok(QueryDeadlocked),
            -440 => Ok(QueryUnterminatedAfterIndefiniteResponse),
            -500 => Ok(PowerOn),
            -600 => Ok(UserRequest),
            -700 => Ok(RequestControl),
            -800 => Ok(OperationComplete),
            _ => Err(value),
        }
    }
}

/// An error code returned by a device
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    NoError,
    Standard(StandardErrorCode),
    Other(i16),
}

impl From<i16> for ErrorCode {
    fn from(value: i16) -> Self {
        if value == 0 {
            ErrorCode::NoError
        } else {
            match StandardErrorCode::try_from(value) {
                Ok(code) => ErrorCode::Standard(code),
                Err(code) => ErrorCode::Other(code),
            }
        }
    }
}

impl From<ErrorCode> for i16 {
    fn from(error: ErrorCode) -> Self {
        match error {
            ErrorCode::NoError => 0,
            ErrorCode::Standard(code) => code as i16,
            ErrorCode::Other(code) => code,
        }
    }
}

/// SCPI 1999.0 error/event queue item
///
/// Returned by error/event queue query (:SYSTem:ERRor:NEXT?).
///
/// Reference: SCPI 1999.0: 21.8 - :ERRor Subsystem
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemErrorResponse {
    pub code: ErrorCode,
    pub message: String,
}

impl ResponseData for SystemErrorResponse {
    fn decode<S: ByteSource>(decoder: &mut Decoder<S>) -> Result<Self, S::Error> {
        let (code, message): (i16, String) = ResponseData::decode(decoder)?;
        Ok(SystemErrorResponse {
            code: ErrorCode::from(code),
            message,
        })
    }
}

/// Represents either a concrete value, or a limit (MIN/MAX).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueOrLimit<T> {
    Value(T),
    Limit(Limit),
}

impl<T> ValueOrLimit<T> {
    pub fn map<O, F: FnOnce(T) -> O>(self, f: F) -> ValueOrLimit<O> {
        use ValueOrLimit::*;
        match self {
            Value(value) => Value(f(value)),
            Limit(limit) => Limit(limit),
        }
    }
}

impl<T> ProgramData for ValueOrLimit<T>
where
    T: ProgramData,
{
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        match self {
            ValueOrLimit::Value(value) => value.encode(encoder),
            ValueOrLimit::Limit(limit) => limit.encode(encoder),
        }
    }
}

/// Represents either a concrete value, or some device default.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueOrDefault<T> {
    Value(T),
    Default,
}

impl<T> ValueOrDefault<T> {
    pub fn map<O, F: FnOnce(T) -> O>(self, f: F) -> ValueOrDefault<O> {
        use ValueOrDefault::*;
        match self {
            Value(value) => Value(f(value)),
            Default => Default,
        }
    }
}

impl<T> From<T> for ValueOrDefault<T> {
    fn from(value: T) -> Self {
        ValueOrDefault::Value(value)
    }
}

impl<T> ProgramData for ValueOrDefault<T>
where
    T: ProgramData,
{
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        match self {
            ValueOrDefault::Value(value) => value.encode(encoder),
            ValueOrDefault::Default => DefaultValue.encode(encoder),
        }
    }
}

/// Represents either a concrete value, a limit (MIN/MAX), or some device default.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueOrDefaultOrLimit<T> {
    Value(T),
    Default,
    Limit(Limit),
}

impl<T> ValueOrDefaultOrLimit<T> {
    pub fn map<O, F: FnOnce(T) -> O>(self, f: F) -> ValueOrDefaultOrLimit<O> {
        use ValueOrDefaultOrLimit::*;
        match self {
            Value(value) => Value(f(value)),
            Limit(limit) => Limit(limit),
            Default => Default,
        }
    }
}

impl<T> From<T> for ValueOrDefaultOrLimit<T> {
    fn from(value: T) -> Self {
        ValueOrDefaultOrLimit::Value(value)
    }
}

impl<T> ProgramData for ValueOrDefaultOrLimit<T>
where
    T: ProgramData,
{
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        match self {
            ValueOrDefaultOrLimit::Value(value) => value.encode(encoder),
            ValueOrDefaultOrLimit::Default => DefaultValue.encode(encoder),
            ValueOrDefaultOrLimit::Limit(limit) => limit.encode(encoder),
        }
    }
}

/// Represents either a limit (MIN/MAX), or some device default.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DefaultOrLimit {
    Default,
    Limit(Limit),
}

impl From<DefaultValue> for DefaultOrLimit {
    fn from(_: DefaultValue) -> Self {
        DefaultOrLimit::Default
    }
}
impl From<Limit> for DefaultOrLimit {
    fn from(limit: Limit) -> Self {
        DefaultOrLimit::Limit(limit)
    }
}

impl ProgramData for DefaultOrLimit {
    fn encode<S: EncodeSink>(&self, encoder: &mut Encoder<S>) -> Result<(), S::Error> {
        match self {
            DefaultOrLimit::Default => DefaultValue.encode(encoder),
            DefaultOrLimit::Limit(limit) => limit.encode(encoder),
        }
    }
}
