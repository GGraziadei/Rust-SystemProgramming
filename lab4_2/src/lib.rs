use std::collections::{HashMap, HashSet, VecDeque};
use crate::CellId::{Compute, Input};

/// `InputCellId` is a unique identifier for an input cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InputCellId(usize);
/// `ComputeCellId` is a unique identifier for a compute cell.
/// Values of type `InputCellId` and `ComputeCellId` should not be mutually assignable,
/// demonstrated by the following tests:
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input: react::ComputeCellId = r.create_input(111);
/// ```
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input = r.create_input(111);
/// let compute: react::InputCellId = r.create_compute(&[react::CellId::Input(input)], |_| 222).unwrap();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ComputeCellId(usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CallbackId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CellId {
    Input(InputCellId),
    Compute(ComputeCellId),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveCallbackError {
    NonexistentCell,
    NonexistentCallback,
}

struct ComputeCell<'fun_l, 'cb_l, T> {
    id : CellId,
    val: Option<T>,
    deps: Vec<CellId>,
    fun: Box<dyn 'fun_l + Fn(&[T]) -> T>,
    callbacks: HashMap<CallbackId, Box<dyn 'cb_l + FnMut(T)>>
}

impl<'fun_l, 'cb_l, T: Copy + PartialEq> ComputeCell<'fun_l, 'cb_l , T> {

    pub fn new(id : ComputeCellId, fun : Box<dyn Fn(&[T]) -> T>, val : T) -> Self
    {
        Self{
            id: Compute(id),
            val: Some(val),
            deps: vec![],
            fun,
            callbacks: Default::default(),
        }
    }

    pub fn add_dep(&mut self, dep_id : CellId )
    {
        self.deps.push(dep_id)
    }

}

struct InputCell<T> {
    id : CellId,
    val: Option<T>,
}

impl<T: Copy + PartialEq> InputCell<T> {
    pub fn new(id : InputCellId, val : T ) -> Self
    {
        Self{
            id : CellId::Input(id),
            val: Some(val),
        }
    }
}
pub struct Reactor<'cb_l, 'fun_l, T> {
    inputs: HashMap<InputCellId, InputCell<T>>,
    cells: HashMap<ComputeCellId, ComputeCell<'fun_l, 'cb_l, T>>,
    inv_deps: HashMap<CellId, HashSet<ComputeCellId>>,
    cb_cells: HashMap<CallbackId, ComputeCellId>,
    cb_ids: usize
}


// You are guaranteed that Reactor will only be tested against types that are Copy + PartialEq.
impl<'fun_l, 'cb_l, T: Copy + PartialEq > Reactor<'fun_l, 'cb_l, T> {
    
    pub fn new()
        -> Self
    {
        Self{
            inputs: HashMap::<InputCellId, InputCell<T>>::new(),
            cells: HashMap::<ComputeCellId, ComputeCell<T>>::new(),
            inv_deps: HashMap::<CellId, HashSet<ComputeCellId>>::new(),
            cb_cells: HashMap::<CallbackId, ComputeCellId>::new(),
            cb_ids: 0,
        }
    }

    fn values(&self, dependencies : &[CellId] )
        -> Result<Box<Vec<T>>, CellId>
    {
        let mut values = VecDeque::<T>::with_capacity(dependencies.len());
        for v in dependencies
        {
            let val = self.value(v.clone());
            if val.is_some(){
                values.push_back(val.unwrap());
            }else {
                return Err(v.clone());
            }
        }

        Ok(Box::new(Vec::<T>::from(values)))
    }


    // Creates an input cell with the specified initial value, returning its ID.
    pub fn create_input(&mut self, _initial: T) -> InputCellId {
        let input_cell_id = InputCellId(self.cb_ids);
        self.cb_ids += 1;
        let input_cell = InputCell::<T>::new(input_cell_id.clone(), _initial);
        self.inputs.insert(input_cell_id , input_cell);
        self.inv_deps.insert(Input(input_cell_id.clone()) , HashSet::<ComputeCellId>::new() );

        input_cell_id
    }

    // Creates a compute cell with the specified dependencies and compute function.
    // The compute function is expected to take in its arguments in the same order as specified in
    // `dependencies`.
    // You do not need to reject compute functions that expect more arguments than there are
    // dependencies (how would you check for this, anyway?).
    //
    // If any dependency doesn't exist, returns an Err with that nonexistent dependency.
    // (If multiple dependencies do not exist, exactly which one is returned is not defined and
    // will not be tested)
    //
    // Notice that there is no way to *remove* a cell.
    // This means that you may assume, without checking, that if the dependencies exist at creation
    // time they will continue to exist as long as the Reactor exists.
    pub fn create_compute< F:'cb_l + 'fun_l +  Fn(&[T]) -> T + 'static>(
        &mut self,
        _dependencies: &[CellId],
        _compute_func: F,
    ) -> Result<ComputeCellId, CellId> {

        let fn_boxed = Box::new(_compute_func);
        let values = *self.values(_dependencies)?;
        let initial_value = fn_boxed(  &values );
        let compute_cell_id = ComputeCellId(self.cb_ids);

        self.cb_ids += 1;

        let mut compute_cell = ComputeCell::<T>::new(compute_cell_id.clone(), fn_boxed, initial_value);
        for dept in _dependencies
        {
            compute_cell.add_dep(dept.clone() );
            self.inv_deps.get_mut(&dept ).unwrap().insert(compute_cell_id.clone() );
        }

        /* Check that thare are not a ciruclar dependecy */
        self.inv_deps.insert(CellId::Compute(compute_cell_id.clone()) , HashSet::<ComputeCellId>::new());
        self.cells.insert(compute_cell_id, compute_cell);

        Ok(compute_cell_id)
    }

    // Retrieves the current value of the cell, or None if the cell does not exist.
    //
    // You may wonder whether it is possible to implement `get(&self, id: CellId) -> Option<&Cell>`
    // and have a `value(&self)` method on `Cell`.
    //
    // It turns out this introduces a significant amount of extra complexity to this exercise.
    // We chose not to cover this here, since this exercise is probably enough work as-is.
    pub fn value(&self, id: CellId) -> Option<T> {

        match id {

            Input(id) => {
                if self.inputs.contains_key(&id){
                    return self.inputs.get(&id).expect("Error").val;
                }
            }

            Compute(id) => {
                if self.cells.contains_key(&id){
                    return self.cells.get(&id).expect("Error").val;
                }
            }

        }

        None
    }

    // Sets the value of the specified input cell.
    //
    // Returns false if the cell does not exist.
    //
    // Similarly, you may wonder about `get_mut(&mut self, id: CellId) -> Option<&mut Cell>`, with
    // a `set_value(&mut self, new_value: T)` method on `Cell`.
    //
    // As before, that turned out to add too much extra complexity.

    pub fn set_value(&mut self, _id: InputCellId, _new_value: T) -> bool {

        if self.inputs.contains_key(&_id)
        {
            self.inputs.get_mut(&_id)
                .expect("Error")
                .val = Some(_new_value);

            let input_cell_id = Input(_id.clone());

            /* During creation of an input cell a new entry is created inside the map */
            assert!(self.inv_deps.contains_key(&input_cell_id ));

            let mut update_queue = VecDeque::<&ComputeCellId>::new();

            for  compute_cell_id in self.inv_deps.get(&input_cell_id )
                .unwrap()
                .iter()
            {
                update_queue.push_front(compute_cell_id);
            }

            while ! update_queue.is_empty() {
                let cell_id = update_queue.pop_back().unwrap();

                let cell_immutable = self.cells.get(cell_id).expect("Error");
                let values = *self.values(&cell_immutable.deps).unwrap();
                let new_val = (cell_immutable.fun)(&values);

                if new_val != cell_immutable.val.unwrap()
                {

                    for _cell_id in self.inv_deps.get(&cell_immutable.id).unwrap()
                    {
                        update_queue.push_front(_cell_id);
                    }

                    let cell_mutable = self.cells.get_mut(cell_id).expect("Error");
                    /*** UpdateValue ***/
                    cell_mutable.val = Some(new_val.clone());

                    for c in cell_mutable.callbacks.values_mut()
                    {
                        c(new_val.clone());
                    }

                }

            }
            
            return true;
        }

        false
    }



    // Adds a callback to the specified compute cell.
    //
    // Returns the ID of the just-added callback, or None if the cell doesn't exist.
    //
    // Callbacks on input cells will not be tested.
    //
    // The semantics of callbacks (as will be tested):
    // For a single set_value call, each compute cell's callbacks should each be called:
    // * Zero times if the compute cell's value did not change as a result of the set_value call.
    // * Exactly once if the compute cell's value changed as a result of the set_value call.
    //   The value passed to the callback should be the final value of the compute cell after the
    //   set_value call.
    pub fn add_callback< F: 'cb_l + 'fun_l + FnMut(T)>(
        &mut self,
        _id: ComputeCellId,
        _callback: F,
    ) -> Option<CallbackId> {

        if ! self.cells.contains_key(&_id){
            return None;
        }

        let callback_id = CallbackId(self.cb_ids + 1 );
        self.cb_ids += 1;

        self.cells.get_mut(&_id).unwrap()
            .callbacks
            .insert(callback_id.clone(), Box::new(_callback) );

        self.cb_cells.insert(callback_id.clone(), _id.clone() );

        Some(callback_id)

    }

    // Removes the specified callback, using an ID returned from add_callback.
    //
    // Returns an Err if either the cell or callback does not exist.
    //
    // A removed callback should no longer be called.
    pub fn remove_callback(
        &mut self,
        cell: ComputeCellId,
        callback: CallbackId,
    ) -> Result<(), RemoveCallbackError> {

        if self.cb_cells.contains_key(&callback){

            if self.cells.contains_key(&cell) {

                let cell = self.cells.get_mut(&cell).unwrap();
                cell.callbacks.remove(&callback);
                self.cb_cells.remove(&callback);

                return Ok(());

            }

            return Err(RemoveCallbackError::NonexistentCell);
        }

        Err(RemoveCallbackError::NonexistentCallback)
    }
}
