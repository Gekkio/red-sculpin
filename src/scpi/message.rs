// SPDX-FileCopyrightText: 2019-2022 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    internal::{declare_tuple_command, declare_tuple_query},
    scpi::types::SystemErrorResponse,
};

// Mandatory SCPI 1999.0 commands

declare_tuple_query! {
    /// SCPI 1999.0 System -\> Error [-\> Next]?
    #[derive(Copy, Clone, Debug)]
    pub struct SystemErrorQuery<":SYST:ERR?", SystemErrorResponse>;
}

declare_tuple_query! {
    /// SCPI 1999.0 System -\> Version?
    #[derive(Copy, Clone, Debug)]
    pub struct SystemVersionQuery<":SYST:VERS?", f32>;
}

declare_tuple_query! {
    /// SCPI 1999.0 Status -\> Operation -\> Event?
    #[derive(Copy, Clone, Debug)]
    pub struct StatusOperationQuery<":STAT:OPER?", u16>;
}

declare_tuple_query! {
    /// SCPI 1999.0 Status -\> Operation -\> Condition?
    #[derive(Copy, Clone, Debug)]
    pub struct StatusOperationConditionQuery<":STAT:OPER:COND?", u16>;
}

declare_tuple_command! {
    /// SCPI 1999.0 Status -\> Operation -\> Enable
    #[derive(Copy, Clone, Debug)]
    pub struct StatusOperationEnable<":STAT:OPER:ENAB">(pub u16);
}

declare_tuple_query! {
    /// SCPI 1999.0 Status -\> Operation -\> Enable?
    #[derive(Copy, Clone, Debug)]
    pub struct StatusOperationEnableQuery<":STAT:OPER:ENAB?", u16>;
}

declare_tuple_query! {
    /// SCPI 1999.0 Status -\> Questionable [-\> Event]?
    #[derive(Copy, Clone, Debug)]
    pub struct StatusQuestionableQuery<":STAT:QUES?", u16>;
}

declare_tuple_query! {
    /// SCPI 1999.0 Status -\> Questionable -\> Condition?
    #[derive(Copy, Clone, Debug)]
    pub struct StatusQuestionableConditionQuery<":STAT:QUES:COND?", u16>;
}

declare_tuple_command! {
    /// SCPI 1999.0 Status -\> Questionable -\> Enable
    #[derive(Copy, Clone, Debug)]
    pub struct StatusQuestionableEnable<":STAT:QUES:ENAB">(pub u16);
}

declare_tuple_query! {
    /// SCPI 1999.0 Status -\> Questionable -\> Enable?
    #[derive(Copy, Clone, Debug)]
    pub struct StatusQuestionableEnableQuery<":STAT:QUES:ENAB?", u16>;
}

declare_tuple_command! {
    /// SCPI 1999.0 Status -\> Preset
    #[derive(Copy, Clone, Debug)]
    pub struct StatusPreset<":STAT:PRES">;
}
