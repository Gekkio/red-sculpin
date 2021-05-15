// SPDX-FileCopyrightText: 2020-2021 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    declare_tuple_command, declare_tuple_query,
    ieee::types::{DeviceIdentification, MacroList, StandardEventStatus, StatusByte},
    ArbitraryAscii, Command,
};

// Mandatory IEEE 488.2 commands

declare_tuple_command! {
    /// IEEE 488.2 Clear Status
    ///
    /// Reference: IEEE 488.2: 10.3 - *CLS, Clear Status Command
    #[derive(Copy, Clone, Debug)]
    pub struct ClearStatus<"*CLS">;
}

declare_tuple_command! {
    /// IEEE 488.2 Standard Event Status Enable
    ///
    /// Reference: IEEE 488.2: 10.10 - *ESE, Standard Event Status Enable Command
    #[derive(Copy, Clone, Debug)]
    pub struct StandardEventStatusEnable<"*ESE">(pub StandardEventStatus);
}

declare_tuple_query! {
    /// IEEE 488.2 Standard Event Status Enable Query
    ///
    /// Reference: IEEE 488.2: 10.11 - *ESE?, Standard Event Status Enable Query
    #[derive(Copy, Clone, Debug)]
    pub struct StandardEventStatusEnableQuery<"*ESE?", StandardEventStatus>;
}

declare_tuple_query! {
    /// IEEE 488.2 Standard Event Status Register Query
    ///
    /// Reference: IEEE 488.2: 10.12 - *ESR?, Standard Event Status Register Query
    #[derive(Copy, Clone, Debug)]
    pub struct StandardEventStatusRegisterQuery<"*ESR?", StandardEventStatus>;
}

declare_tuple_query! {
    /// IEEE 488.2 Identification Query
    ///
    /// Reference: IEEE 488.2: 10.14 - *IDN?, Identification Query
    #[derive(Copy, Clone, Debug)]
    pub struct IdentificationQuery<"*IDN?", DeviceIdentification>;
}

declare_tuple_command! {
    /// IEEE 488.2 Operation Complete
    ///
    /// Reference: IEEE 488.2: 10.18 - *OPC, Operation Complete Command
    #[derive(Copy, Clone, Debug)]
    pub struct OperationComplete<"*OPC">;
}

declare_tuple_query! {
    /// IEEE 488.2 Operation Complete Query
    ///
    /// Reference: IEEE 488.2: 10.19 - *OPC?, Operation Complete Query
    #[derive(Copy, Clone, Debug)]
    pub struct OperationCompleteQuery<"*OPC?", bool>;
}

declare_tuple_command! {
    /// IEEE 488.2 Reset
    ///
    /// Reference: IEEE 488.2: 10.32 - *RST, Reset Command
    #[derive(Copy, Clone, Debug)]
    pub struct Reset<"*RST">;
}

declare_tuple_command! {
    /// IEEE 488.2 Service Request Enable
    ///
    /// Reference: IEEE 488.2: 10.34 - *SRE, Service Request Enable Command
    #[derive(Copy, Clone, Debug)]
    pub struct ServiceRequestEnable<"*SRE">(pub StatusByte);
}

declare_tuple_query! {
    /// IEEE 488.2 Service Request Enable Query
    ///
    /// Reference: IEEE 488.2: 10.35 - *SRE?, Service Request Enable Query
    #[derive(Copy, Clone, Debug)]
    pub struct ServiceRequestEnableQuery<"*SRE?", StatusByte>;
}

declare_tuple_query! {
    /// IEEE 488.2 Status Byte Query
    ///
    /// Reference: IEEE 488.2: 10.36 - *STB?, Read Status Byte Query
    #[derive(Copy, Clone, Debug)]
    pub struct StatusByteQuery<"*STB?", StatusByte>;
}

declare_tuple_query! {
    /// IEEE 488.2 Test Query
    ///
    /// Reference: IEEE: 488.2: 10.38 - *TST?, Self-Test Query
    #[derive(Copy, Clone, Debug)]
    pub struct TestQuery<"*TST?", bool>;
}

declare_tuple_command! {
    /// IEEE 488.2 Wait
    ///
    /// Reference: IEEE 488.2: 10.39 - *WAI, Wait-to-Continue Command
    #[derive(Copy, Clone, Debug)]
    pub struct Wait<"*WAI">;
}

// Optional IEEE 488.2 commands

declare_tuple_command! {
    /// IEEE 488.2 Accept Address
    ///
    /// Reference: IEEE 488.2: 10.1 - *AAD, Accept Address Command
    #[derive(Copy, Clone, Debug)]
    pub struct AcceptAddress<"*AAD">;
}

declare_tuple_query! {
    /// IEEE 488.2 Calibration Query
    ///
    /// Reference: IEEE 488.2: 10.2 - *CAL?, Calibration Query
    #[derive(Copy, Clone, Debug)]
    pub struct CalibrationQuery<"CAL?", i16>;
}

declare_tuple_command! {
    /// IEEE 488.2 Define Device Trigger
    ///
    /// Reference: IEEE 488.2: 10.4 - *DDT, Define Device Trigger Command
    #[derive(Copy, Clone, Debug)]
    pub struct DefineDeviceTrigger<'a, "DDT">(pub &'a [u8]);
}

declare_tuple_query! {
    /// IEEE 488.2 Define Device Trigger Query
    ///
    /// Reference: IEEE 488.2: 10.5 - *DDT?, Define Device Trigger Query
    #[derive(Copy, Clone, Debug)]
    pub struct DefineDeviceTriggerQuery<"DDT?", Vec<u8>>;
}

declare_tuple_command! {
    /// IEEE 488.2 Disable Listener Function
    ///
    /// Reference: IEEE 488.2: 10.6 - *DLF, Disable Listener Function Command
    #[derive(Copy, Clone, Debug)]
    pub struct DisableListenerFunction<"*DLF">;
}

/// IEEE 488.2 Define Macro
///
/// Reference: IEEE 488.2: 10.7 - *DMC, Define Macro Command
#[derive(Copy, Clone, Debug)]
pub struct DefineMacro<'a> {
    name: &'a str,
    data: &'a [u8],
}

impl<'a> Command for DefineMacro<'a> {
    type ProgramData = (&'a str, &'a [u8]);

    fn mnemonic(&self) -> &str {
        "*DMC"
    }

    fn program_data(&self) -> Option<Self::ProgramData> {
        Some((self.name, self.data))
    }
}

declare_tuple_command! {
    /// IEEE 488.2 Enable Macros
    ///
    /// Reference: IEEE 488.2: 10.8 - *EMC, Enable Macro Command
    #[derive(Copy, Clone, Debug)]
    pub struct EnableMacros<"*EMC">(pub bool);
}

declare_tuple_query! {
    /// IEEE 488.2 Enable Macros Query
    ///
    /// Reference: IEEE 488.2: 10.9 - *EMC?, Enable Macro Query
    #[derive(Copy, Clone, Debug)]
    pub struct EnableMacrosQuery<"*EMC?", bool>;
}

declare_tuple_query! {
    /// IEEE 488.2 Get Macro Contents Query
    ///
    /// Reference: IEEE 488.2: 10.13 - *GMC?, Get Macro Contents Query
    #[derive(Copy, Clone, Debug)]
    pub struct GetMacroContentsQuery<'a, "*GMC?", Vec<u8>>(pub &'a str);
}

declare_tuple_query! {
    /// IEEE 488.2 Individual Status Query
    ///
    /// Reference: IEEE 488.2: 10.15 - *IST?, Individual Status Query
    #[derive(Copy, Clone, Debug)]
    pub struct IndividualStatusQuery<"*IST?", bool>;
}

declare_tuple_query! {
    /// IEEE 488.2 Learn Macro Query
    ///
    /// Reference: IEEE 488.2: 10.16 - *LMC?, Learn Macro Query
    #[derive(Copy, Clone, Debug)]
    pub struct LearnMacroQuery<"*LMC?", MacroList>;
}

declare_tuple_query! {
    /// IEEE 488.2 Option Identification Query
    ///
    /// Reference: IEEE 488.2: 10.20 - *LMC?, Option Identification Query
    #[derive(Copy, Clone, Debug)]
    pub struct OptionIdentificationQuery<"*OPT?", ArbitraryAscii>;
}

/// IEEE 488.2 Pass Control Back
///
/// Reference: IEEE 488.2: 10.21 - *PCB, Pass Control Back
#[derive(Copy, Clone, Debug)]
pub struct PassControlBack {
    primary_addr: u32,
    secondary_addr: Option<u32>,
}

impl Command for PassControlBack {
    type ProgramData = (u32, Option<u32>);

    fn mnemonic(&self) -> &str {
        "*PCB"
    }

    fn program_data(&self) -> Option<Self::ProgramData> {
        Some((self.primary_addr, self.secondary_addr))
    }
}

declare_tuple_command! {
    /// IEEE 488.2 Purge Macros
    ///
    /// Reference: IEEE 488.2: 10.22 - *PMC, Purge Macros Command
    #[derive(Copy, Clone, Debug)]
    pub struct PurgeMacros<"*PMC">;
}

declare_tuple_command! {
    /// IEEE 488.2 Parallel Poll Enable Register
    ///
    /// Reference: IEEE 488.2: 10.23 - *PRE, Parallel Poll Enable Register Command
    #[derive(Copy, Clone, Debug)]
    pub struct ParallelPollEnableRegisterCommand<"*PRE">(pub u16);
}

declare_tuple_query! {
    /// IEEE 488.2 Parallel Poll Enable Register Query
    ///
    /// Reference: IEEE 488.2: 10.24 - *PRE?, Parallel Poll Enable Register Query
    #[derive(Copy, Clone, Debug)]
    pub struct ParallelPollEnableRegisterQuery<"*PRE?", u16>;
}

declare_tuple_command! {
    /// IEEE 488.2 Power-On Status Clear
    ///
    /// Reference: IEEE 488.2: 10.25 - *PSC, Power-On Status Clear Command
    #[derive(Copy, Clone, Debug)]
    pub struct PowerOnStatusClear<"*PSC">(pub bool);
}

declare_tuple_query! {
    /// IEEE 488.2 Power-On Status Clear Query
    ///
    /// Reference: IEEE 488.2: 10.26 - *PSC?, Power-On Status Clear Query
    #[derive(Copy, Clone, Debug)]
    pub struct PowerOnStatusClearQuery<"*PSC?", bool>;
}

declare_tuple_command! {
    /// IEEE 488.2 Protected User Data
    ///
    /// Reference: IEEE 488.2: 10.27 - *PUD, Protected User Data Command
    #[derive(Copy, Clone, Debug)]
    pub struct ProtectedUserData<'a, "*PUD">(pub &'a [u8]);
}

declare_tuple_query! {
    /// IEEE 488.2 Protected User Data Query
    ///
    /// Reference: IEEE 488.2: 10.28 - *PUD?, Protected User Data Query
    #[derive(Copy, Clone, Debug)]
    pub struct ProtectedUserDataQuery<"*PUD?", Vec<u8>>;
}

declare_tuple_command! {
    /// IEEE 488.2 Recall
    ///
    /// Reference: IEEE 488.2: 10.29 - *RCL, Recall Command
    #[derive(Copy, Clone, Debug)]
    pub struct Recall<"*RCL">(pub u32);
}

declare_tuple_command! {
    /// IEEE 488.2 Resource Description Transfer
    ///
    /// Reference: IEEE 488.2: 10.30 - *PUD, Resource Description Transfer Command
    #[derive(Copy, Clone, Debug)]
    pub struct ResourceDescriptionTransfer<'a, "*RDT">(pub &'a [u8]);
}

declare_tuple_query! {
    /// IEEE 488.2 Resource Description Transfer Query
    ///
    /// Reference: IEEE 488.2: 10.31 - *RDT?, Resource Description Transfer Query
    #[derive(Copy, Clone, Debug)]
    pub struct ResourceDescriptionTransferQuery<"*RDT?", Vec<u8>>;
}

declare_tuple_command! {
    /// IEEE 488.2 Save
    ///
    /// Reference: IEEE 488.2: 10.33 - *SAV, Save Command
    #[derive(Copy, Clone, Debug)]
    pub struct Save<"*SAV">(pub u32);
}

declare_tuple_command! {
    /// IEEE 488.2 Trigger
    ///
    /// Reference: IEEE 488.2: 10.37 - *TRG, Trigger Command
    #[derive(Copy, Clone, Debug)]
    pub struct Trigger<"*TRG">;
}

declare_tuple_command! {
    /// IEEE 488.2 Remove Individual Macro
    ///
    /// Reference: IEEE 488.2: 10.40 - *RMC, Remove Individual Macro Command
    #[derive(Copy, Clone, Debug)]
    pub struct RemoveIndividualMacro<'a, "*RMC">(pub &'a str);
}

declare_tuple_command! {
    /// IEEE 488.2 Save Default Device Settings
    ///
    /// Reference: IEEE 488.2: 10.41 - *SDS, Save Default Device Settings Command
    #[derive(Copy, Clone, Debug)]
    pub struct SaveDefaultDeviceSettings<"*SDS">(pub u32);
}
