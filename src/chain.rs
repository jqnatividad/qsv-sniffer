
// states: 0 = SteadyStrict, 1 = SteadyFlexible, 2 = Unsteady
pub(crate) const N_STATES: usize = 3;
pub(crate) const STATE_STEADYSTRICT: usize = 0;
pub(crate) const STATE_STEADYFLEX: usize = 1;
pub(crate) const STATE_UNSTEADY: usize = 2;
// observations: 0 = MaxValue, 1 = Other, 2 = Zero
pub(crate) const N_OBS: usize = 3;
pub(crate) const OBS_MAXVALUE: usize = 0;
pub(crate) const OBS_OTHER: usize = 1;
pub(crate) const OBS_ZERO: usize = 2;

#[derive(Debug, Default, Clone)]
pub(crate) struct Chain {
    observations: Vec<usize>,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct VIteration {
    pub(crate) prob: f64,
    pub(crate) prev: Option<usize>,
}
#[derive(Debug, Clone)]
pub(crate) struct ViterbiResults {
    pub(crate) max_delim_freq: usize,
    pub(crate) path: Vec<(usize, VIteration)>
}
impl Chain {
    pub(crate) fn add_observation(&mut self, obs: usize) {
        self.observations.push(obs);
    }
    pub(crate) fn viterbi(&mut self) -> ViterbiResults {
        if self.observations.is_empty() {
            return ViterbiResults { max_delim_freq: 0, path: vec![] };
        }
        // compute the max frequency value; unwrap is safe, we just checked if vector is empty
        let max_value = *self.observations.iter().max().unwrap();
        if max_value == 0 {
            // no frequencies observed! return unsteady state
            return ViterbiResults {
                max_delim_freq: max_value,
                path: vec![(STATE_UNSTEADY, VIteration { prob: 0.0, prev: Some(STATE_UNSTEADY) })]
            }
        }

        let start_prob = [
            /*SteadyStrict*/    1.0 / 3.0,
            /*SteadyFlexible*/  1.0 / 3.0,
            /*Unsteady*/        1.0 / 3.0,
        ];
        let mut trans_prob = [   /* ToSteadyStrict  ToSteadyFlexible    ToUnsteady */
        /*FromSteadyStrict*/        1.0,            0.0,                0.0,
        /*FromSteadyFlexible*/      0.0,            1.0,                0.0,
        /*FromUnsteady*/            0.2,            0.2,                0.6,
        ];
        let update_trans_prob = |trans_prob: &mut [f64; N_STATES * N_STATES]| {
            const DELTA: f64 = 0.01;

            // decrement transition from Unsteady to either Steady state by delta
            trans_prob[STATE_UNSTEADY * N_STATES + STATE_STEADYSTRICT] =
                (trans_prob[STATE_UNSTEADY * N_STATES + STATE_STEADYSTRICT] - DELTA).max(0.0);
            trans_prob[STATE_UNSTEADY * N_STATES + STATE_STEADYFLEX] =
                (trans_prob[STATE_UNSTEADY * N_STATES + STATE_STEADYFLEX] - DELTA).max(0.0);
            // increment transition from Unsteady to itself by 2*delta
            trans_prob[STATE_UNSTEADY * N_STATES + STATE_UNSTEADY] =
                (trans_prob[STATE_UNSTEADY * N_STATES + STATE_UNSTEADY] + 2.0 * DELTA).min(1.0);
        };

        let emit_uniprob = 1.0 / (max_value as f64 + 1.0);
        let emit_prob = [    /* MaxValue        Other                       Zero*/
        /*FromSteadyStrict*/    1.0,            0.0,                        0.0,
        /*FromSteadyFlexible*/  0.7,            0.3,                        0.0,
        /*FromUnsteady*/        emit_uniprob,   1.0 - 2.0 * emit_uniprob,   emit_uniprob
        ];
        // function to map frequency to observation
        let map_observation = |freq: usize| {
            if freq == max_value {
                OBS_MAXVALUE
            } else if freq == 0 {
                OBS_ZERO
            } else {
                OBS_OTHER
            }
        };

        let mut iterations: Vec<Vec<VIteration>> = vec![vec![]];
        #[allow(clippy::needless_range_loop)]
        for state_idx in 0..N_STATES {
            iterations[0].push(VIteration {
                prob: start_prob[state_idx],
                prev: None,
            });
            // print!("{:>30e},{}", iterations[0][iterations[0].len() - 1].prob, " ");
        }
        // println!();
        for t in 0..self.observations.len() {
            // since we start with iterations already at length 1, the index of this newly-pushed
            // vector will be t + 1.
            iterations.push(vec![]);
            for state_idx in 0..N_STATES {
                let (max_prev_st, max_tr_prob) = (0..N_STATES).fold((None, 0.0),
                    |acc, prev_state_idx| {
                        let tr_prob = iterations[t][prev_state_idx].prob
                            * trans_prob[prev_state_idx * N_STATES + state_idx];
                        if acc.0.is_none() || tr_prob > acc.1 {
                            (Some(prev_state_idx), tr_prob)
                        } else {
                            acc
                        }
                    }
                );
                assert!(max_prev_st.is_some(), "All previous states at 0.0 probability");
                iterations[t + 1].push(VIteration {
                    prob: max_tr_prob * emit_prob[state_idx * N_OBS
                        + map_observation(self.observations[t])],
                    prev: max_prev_st
                });
                update_trans_prob(&mut trans_prob);
                // print!("{:>30e},{}", iterations[t][iterations[t].len() - 1].prob,
                //     iterations[t][iterations[t].len() - 1].prev.unwrap(),);
            }
            // println!();
        }

        let (final_state, final_viter) = iterations[iterations.len()-1].iter().enumerate()
            .fold((0, None), |acc: (usize, Option<VIteration>), (state, &viter)| {
                match acc.1 {
                    Some(max_viter) => if viter.prob > max_viter.prob {
                        (state, Some(viter))
                    } else {
                        acc
                    },
                    None => (state, Some(viter))
                }
            }
        );
        let final_viter = final_viter.expect("All final states at 0.0 probability");
        let mut path = vec![(final_state, final_viter)];
        for t in (-1isize..iterations.len() as isize - 2).rev() {
            let prev_viter = path[path.len() - 1].1;
            let prev_state = prev_viter.prev
                .expect("all iterations should have a previous state except initial iteration");
            path.push((prev_state, iterations[(t + 1) as usize][prev_state]));
        }
        path.reverse();
        ViterbiResults { max_delim_freq: max_value, path }
    }
}
