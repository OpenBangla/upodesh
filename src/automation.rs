use regex_automata::{dfa::{dense::DFA, Automaton}, util::{primitives::StateID, start::Config}, Anchored};

pub struct DfaFstAutomaton(DFA<Vec<u32>>);

impl DfaFstAutomaton {
    pub fn new(regex: &str) -> Self {
        let dfa = DFA::new(regex).unwrap();
        Self(dfa)
    }
}

impl fst::Automaton for DfaFstAutomaton {
    type State = StateID;

    #[inline]
    fn start(&self) -> Self::State {
        let config = Config::new().anchored(Anchored::Yes);

        self.0.start_state(&config).unwrap()
    }

    #[inline]
    fn is_match(&self, state: &Self::State) -> bool {
        self.0.is_match_state(*state)
    }

    #[inline]
    fn can_match(&self, state: &Self::State) -> bool {
        !self.0.is_dead_state(*state)
    }

    #[inline]
    fn accept_eof(&self, state: &StateID) -> Option<StateID> {
        if self.0.is_match_state(*state) {
            return Some(*state);
        }
        Some(self.0.next_eoi_state(*state))
    }

    #[inline]
    fn accept(&self, state: &Self::State, byte: u8) -> Self::State {
        if self.0.is_match_state(*state) {
            return *state;
        }
        self.0.next_state(*state, byte)
    }
}
