macro_rules! substance_store_wrapper {
    ( $($field_path:ident).+, $($id_map_path:ident).+ ) => {
        /// Retrieves the current simulation time for the store.
        pub fn sim_time(&self) -> crate::sim::SimTime {
            self.$($field_path).+.sim_time()
        }

        /// Retrieves the concentration of a given Substance in the store.
        ///
        /// ### Arguments
        /// * `substance` - Substance to retrieve
        ///
        /// Returns the current concentration of the substance
        pub fn concentration_of(&self, substance: &crate::substance::Substance) -> crate::substance::SubstanceConcentration {
            self.$($field_path).+.concentration_of(substance)
        }

        /// Get a reference to a previously added `SubstanceChange`
        ///
        /// ### Arguments
        /// * `change_id`  - change id returned previously
        ///
        /// Returns a reference to the `SubstanceChange`
        pub(crate) fn get_substance_change<'a>(
            &'a self,
            substance: &crate::substance::Substance,
            change_id: &crate::util::IdType,
        ) -> Option<&'a crate::substance::SubstanceChange> {
            self.$($field_path).+.get_substance_change(substance, change_id)
        }

        /// Get all previously scheduled `SubstanceChange` objects
        /// for the given `Substance`
        ///
        /// ### Arguments
        /// * `substance`  - the substance which was changed
        ///
        /// Returns an iterator of (change_id, substance_change) references
        pub(crate) fn get_substance_changes<'a>(
            &'a self,
            substance: crate::substance::Substance,
        ) -> impl Iterator<Item = (&'a IdType, &'a crate::substance::SubstanceChange)> {
            match self.$($id_map_path).+.get(&substance) {
                Some(list) => {
                    let iter = list.iter()
                        .map(move |id| (id, self.$($field_path).+.get_substance_change(&substance, id).unwrap()));
                    either::Left(iter)
                }
                None => either::Right(std::iter::empty())
            }
        }

        /// Get all previously scheduled change ids
        /// for the given `Substance`
        ///
        /// ### Arguments
        /// * `substance`  - the substance which was changed
        ///
        /// Returns an iterator of change ids
        pub(crate) fn get_substance_change_ids<'a>(
            &'a self,
            substance: crate::substance::Substance,
        ) -> impl Iterator<Item = &'a IdType> {
            match self.$($id_map_path).+.get(&substance) {
                Some(list) => either::Left(list.iter()),
                None => either::Right(std::iter::empty())
            }
        }

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
        /// * `delay`      - delay in simulation time before starting the change
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
            let id = self.$($field_path).+.schedule_change(substance, amount, delay, duration, bound_fn);
            self.$($id_map_path).+.entry(substance).or_default().push(id);
            id
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
            self.$($id_map_path).+.entry(*substance).or_default().retain(|c| c != change_id);
            self.$($field_path).+.unschedule_change(substance, change_id)
        }
    };
}

pub(crate) use substance_store_wrapper;
