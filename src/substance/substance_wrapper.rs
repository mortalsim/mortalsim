
macro_rules! substance_store_wrapper {
    ( $($field_path:ident).+ ) => {
            /// Schedule a substance change on a given Vessel
            /// with a sigmoid shape over the given duration,
            /// startinig immediately.
            /// 
            /// Panics if `duration <= 0`
            ///
            /// ### Arguments
            /// * `substance`  - the substance to change
            /// * `amount`     - total concentration change to take place
            /// * `duration`   - amount of time over which the change takes place
            ///
            /// Returns an id corresponding to this change, if successful
            pub fn schedule_change(
                &mut self,
                substance: crate::substance::Substance,
                amount: crate::substance::SubstanceConcentration,
                duration: crate::sim::SimTime
            ) -> IdType {
                self.schedule_custom_change(substance, amount, crate::sim::SimTime::from_s(0.0), duration, crate::util::BoundFn::Sigmoid)
            }

            /// Schedule a substance change on the store
            /// with a custom shape over the given duration.
            ///
            /// Panics if `delay < 0` or `duration <= 0`
            ///
            /// ### Arguments
            /// * `substance`  - the substance to change
            /// * `delay`      - future simulation time to start the change
            /// * `substance`  - the substance to change
            /// * `amount`     - total concentration change to take place
            /// * `duration`   - amount of time over which the change takes place
            /// * `bound_fn`   - the shape of the function
            ///
            /// Returns an id corresponding to this change
            pub fn schedule_custom_change(
                &mut self,
                substance: crate::substance::Substance,
                amount: crate::substance::SubstanceConcentration,
                delay: crate::sim::SimTime,
                duration: crate::sim::SimTime,
                bound_fn: crate::util::BoundFn,
            ) -> IdType {
                self.$($field_path).+.schedule_change(substance, amount, delay, duration, bound_fn)
            }

            /// Schedule a substance change on this store
            /// defined by the given `SubstanceChange`
            ///
            /// ### Arguments
            /// * `substance`  - the substance to change
            /// * `change`     - the change to exert on the substance
            ///
            /// Returns an id corresponding to this change
            pub(crate) fn schedule_substance_change(
                &mut self,
                substance: Substance,
                change: SubstanceChange
            ) -> IdType {
                self.$($field_path).+.schedule_substance_change(substance, change)
            }

            /// Unschedule a substance change on this store
            ///
            /// ### Arguments
            /// * `substance` - the substance which was scheduled to be changed
            /// * `change_id` - the id returned from the call to schedule_change
            ///
            /// Returns a `SubstanceChange` if found and the change hasn't completed, None otherwise
            pub fn unschedule_change(
                &mut self,
                substance: &crate::substance::Substance,
                change_id: &crate::util::IdType,
            ) -> Option<crate::substance::SubstanceChange> {
                self.$($field_path).+.unschedule_change(substance, change_id)
            }
    };
}

pub(crate) use substance_store_wrapper;
