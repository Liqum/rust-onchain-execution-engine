// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use ink_core::env::EnvTypes;
use scale::{Codec, Decode, Encode};
use pallet_indices::address::Address;
use sp_runtime::traits::Member;
use crate::{AccountId, AccountIndex, Balance, NodeRuntimeTypes};

/// Default runtime Call type, a subset of the runtime Call module variants
///
/// The codec indices of the  modules *MUST* match those in the concrete runtime.
#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
pub enum Call {
    #[codec(index = "6")]
    BpmnInterpreter(BpmnInterpreter),
}

impl From<BpmnInterpreter> for Call {
    fn from(bpmn_interpreter_call: BpmnInterpreter) -> Call {
        Call::BpmnInterpreter(bpmn_interpreter_call)
    }
}
/// Generic Balance Call, could be used with other runtimes
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum BpmnInterpreter
{
    #[allow(non_camel_case_types)]
    continue_execution(u64, u128),
}

/// Construct a `BpmnInterpreter::continue_execution` call
pub fn continue_execution(instance_id: u64, element_index: u128) -> Call {
    BpmnInterpreter::continue_execution(instance_id, element_index).into()
}

#[cfg(test)]
mod tests {
    use crate::{calls, NodeRuntimeTypes};
    use super::Call;

    use node_runtime::{self, Runtime, InstanceId};
    use pallet_indices::address;
    use scale::{Decode, Encode};


    #[test]
    fn call_continue_execution() {
        let element_index = 0;
        let instance_id: u64 = 0;

        let contract_continue_execution =
            calls::BpmnInterpreter::continue_execution(instance_id, element_index);
        let contract_call = Call::BpmnInterpreter(contract_continue_execution);

        // let srml_continue_execution = node_runtime::BpmnInterpreterCall::<Runtime>::continue_execution(instance_id, element_index);
        let srml_continue_execution = node_runtime::BpmnInterpreterCall::continue_execution(instance_id, element_index);
        let srml_call = node_runtime::Call::BpmnInterpreter(srml_continue_execution);

        let contract_call_encoded = contract_call.encode();
        let srml_call_encoded = srml_call.encode();

        //assert_eq!(srml_call_encoded, contract_call_encoded);

        let srml_call_decoded: node_runtime::Call =
            Decode::decode(&mut contract_call_encoded.as_slice())
                .expect("BpmnInterpreter continue_execution call decodes to srml type");
        let srml_call_encoded = srml_call_decoded.encode();
        let contract_call_decoded: Call = Decode::decode(&mut contract_call_encoded.as_slice())
            .expect("BpmnInterpreter continue_execution call decodes back to contract type");
        assert!(contract_call == contract_call_decoded);
    }
}
