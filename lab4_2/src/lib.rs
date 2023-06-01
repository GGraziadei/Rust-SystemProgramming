use std::collections::{HashMap, HashSet};
use std::process::id;
use crate::CellId::Compute;

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

struct ComputeCell<T> {
    id : CellId,
    val: Option<T>,
    deps: Vec<CellId>,
    fun: Box<dyn Fn(&[T]) -> T>,
    callbacks: HashMap<CallbackId, Box<dyn FnMut(T)>>
}

impl<T: Copy + PartialEq> ComputeCell<T> {

    pub fn new(id : ComputeCellId, fun : Box<dyn Fn(&[T]) -> T>, t: T) -> Self
    {
        Self{
            id: Compute(id),
            val: Some(t),
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
pub struct Reactor<T> {
    inputs: Vec<InputCell<T>>,
    cells: Vec<ComputeCell<T>>,
    inv_deps: HashMap<CellId, HashSet<ComputeCellId>>,
    cb_cells: HashMap<CallbackId, ComputeCellId>,
    cb_ids: usize
}


// You are guaranteed that Reactor will only be tested against types that are Copy + PartialEq.
impl<T: Copy + PartialEq> Reactor<T> {
    
    pub fn new() -> Self {
        Self{
            inputs: vec![],
            cells: vec![],
            inv_deps: HashMap::<CellId, HashSet<ComputeCellId>>::new(),
            cb_cells: HashMap::<CallbackId, ComputeCellId>::new(),
            cb_ids: 0,
        }
    }

    // Creates an input cell with the specified initial value, returning its ID.
    pub fn create_input(&mut self, _initial: T) -> InputCellId {
        let input_cell_id = InputCellId(self.cb_ids);
        self.cb_ids += 1;
        let input_cell = InputCell::<T>::new(input_cell_id.clone(), _initial);
        self.inputs.push(input_cell);
        self.inv_deps.insert(CellId::Input(input_cell_id.clone()) , HashSet::<ComputeCellId>::new());

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
    pub fn create_compute<F: Fn(&[T]) -> T + 'static>(
        &mut self,
        _dependencies: &[CellId],
        _compute_func: F,
    ) -> Result<ComputeCellId, CellId> {

        let fn_boxed = Box::new(_compute_func);
        let compute_cell_id = ComputeCellId(self.cb_ids);
        self.cb_ids += 1;

        let mut compute_cell = ComputeCell::<T>::new(compute_cell_id.clone(), fn_boxed, _compute_func(_dependencies));


        for dept in _dependencies
        {
            compute_cell.add_dep(dept.clone() );

            if self.inv_deps.contains_key(dept){
                if let Compute(mut id) = dept.clone() {
                    self.inv_deps.get_mut(dept ).unwrap().insert(id);
                }
            }else {
                return  Err(dept.clone());
            }

        }

        /* Check that thare are not a ciruclar dependecy */
        self.inv_deps.insert(CellId::Compute(compute_cell_id.clone()) , HashSet::<ComputeCellId>::new());
        self.cells.push(compute_cell);

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

        for input_cell in self.inputs.iter(){
            if input_cell.id == id {
                return  input_cell.val.clone();
            }
        }

        for compute_cell in self.cells.iter(){
            if compute_cell.id == id {
                return  compute_cell.val.clone();
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

        for input_cell in self.inputs.iter_mut(){
            if input_cell.id == CellId::Input(_id) {
                input_cell.val = Some(_new_value);
                return true;
            }
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
    pub fn add_callback<F: FnMut(T)>(
        &mut self,
        _id: ComputeCellId,
        _callback: F,
    ) -> Option<CallbackId> {
        unimplemented!()
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
        unimplemented!(
            "Remove the callback identified by the CallbackId {callback:?} from the cell {cell:?}"
        )
    }
}
