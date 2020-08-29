// If the stored type is clone, then implement clone for ReactiveStateAccess
pub mod atom;
pub mod atom_undo;
pub mod reaction;

pub trait CloneReactiveState<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;
    fn soft_get(&self) -> Option<T>;
}

pub trait ObserveChangeReactiveState<T>
where
    T: Clone + 'static + PartialEq,
{
    fn observe_change(&self) -> (Option<T>, T);
    fn has_changed(&self) -> bool;
    fn on_change<F: FnOnce(&T, &T) -> R, R>(&self, func: F) -> R;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        reactive_state_access::{atom::Atom, atom_undo::AtomUndo, reaction::Reaction},
        *,
    };
    use atomic_hooks_macros::*;
    use store::RxFunc;

    #[atom]
    fn a() -> Atom<i32> {
        0
    }

    #[atom]
    fn b() -> Atom<i32> {
        0
    }

    #[reaction]
    fn a_b_subtraction() -> Reaction<i32> {
        let a = a().observe();
        let b = b().observe();
        (a - b)
    }

    #[reaction]
    fn a_b_reversible_subtraction() -> Reaction<i32> {
        let a = a_reversible().observe();
        let b = b_reversible().observe();
        (a - b)
    }

    #[atom]
    fn c() -> Atom<i32> {
        0
    }

    #[reaction]
    fn count_print_when_update() -> Reaction<i32> {
        let c = c();
        let update = a().on_update(|| {
            println!("UPDATE !!!");
            c.update(|mut v| *v = *v + 1)
        });
        c.get()
    }

    #[reaction]
    fn count_subtraction_when_update() -> Reaction<i32> {
        let c = c();
        let update = a_b_subtraction().on_update(|| {
            println!("UPDATE !!!");
            c.update(|mut v| *v = *v + 1)
        });
        c.get()
    }

    #[atom(undo)]
    fn a_reversible() -> AtomUndo<i32> {
        0
    }

    #[atom(undo)]
    fn b_reversible() -> AtomUndo<i32> {
        0
    }
    #[test]
    fn test_on_changes_on_reaction() {
        let a_b_subtraction = a_b_subtraction();
        let mut previous = 99;
        let mut current = 99;
        a_b_subtraction.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0); //todo : should we expect None when init ?
        assert_eq!(current, 0);
        a().set(1);
        a_b_subtraction.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0); //todo : should we expect None when init ?
        assert_eq!(current, 1);
        a().set(2);
        a_b_subtraction.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 1); //todo : should we expect None when init ?
        assert_eq!(current, 2);
    }
    #[test]
    fn test_has_changes_on_reaction() {
        let a_b_subtraction = a_b_subtraction();

        a().set(2);
        let changes_happened = a_b_subtraction.has_changed();
        assert_eq!(changes_happened, true);

        a().set(3);
        let changes_happened = a_b_subtraction.has_changed();
        assert_eq!(changes_happened, true);

        a().set(3);
        let changes_happened = a_b_subtraction.has_changed();
        assert_eq!(changes_happened, false);
    }
    #[test]
    fn test_observe_changes_on_reaction() {
        let a_b_subtraction = a_b_subtraction();
        let changes = a_b_subtraction.observe_change();
        assert_eq!(changes.0.is_none(), true);
        assert_eq!(changes.1, 0);

        a().set(2);
        let changes = a_b_subtraction.observe_change();
        assert_eq!(changes.0.unwrap(), 1);
        assert_eq!(changes.1, 2);
    }

    #[test]
    fn test_observe_on_atom() {
        let a = a();
        let change = a.observe_change();
        println!("{:?}", change.0);
        println!("{:?}", change.1);
        assert_eq!(change.0.is_none(), true);
        assert_eq!(change.1, 0);
        a.set(1);
        let change2 = a.observe_change();
        println!("{:?}", change2.0);
        println!("{:?}", change2.1);
        assert_eq!(change2.0.unwrap(), 0);
        assert_eq!(change2.1, 1);
    }
    #[test]
    fn test_has_changed_on_atom() {
        let a = a();
        a.set(1);

        assert_eq!(a.has_changed(), true);
        a.set(1);
        assert_eq!(a.has_changed(), false);
    }
    #[test]
    fn test_on_changes_on_atom() {
        let a = a();
        let mut previous = 99;
        let mut current = 99;
        a.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0); //todo : should we expect None when init ?
        assert_eq!(current, 0);
        a.set(1);
        a.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0);
        assert_eq!(current, 1);
        a.set(1);
        a.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 0);
        assert_eq!(current, 1);
        a.set(2);
        a.on_change(|p, c| {
            previous = *p;
            current = *c;
        });
        assert_eq!(previous, 1, "we should get 1");
        assert_eq!(current, 2, "we should get 2");
    }

    #[test]
    fn test_get_with() {
        let a_b_reversible_subtraction = a_b_reversible_subtraction();
        a_reversible().set(3);
        b_reversible().set(5);

        a_reversible().get_with(|v| assert_eq!(v, &3, "We should get 3"));
        b_reversible().get_with(|v| assert_eq!(v, &5, "We should get 5"));
        a_b_reversible_subtraction.get_with(|v| assert_eq!(v, &-2, "We should get -2"));
        a().set(3);
        b().set(5);

        a().get_with(|v| assert_eq!(v, &3, "We should get 3"));
        b().get_with(|v| assert_eq!(v, &5, "We should get 5"));
    }

    #[test]
    fn test_on_update() {
        let print = count_print_when_update();
        a().update(|v| *v = 32);
        a().set(2);
        a().set(25);
        a().set(1);

        println!("{:?}", print.get());

        assert_eq!(print.get(), 5)
    }

    #[test]
    fn test_on_update_reaction() {
        let count = count_subtraction_when_update();
        println!("{:?}", a_b_subtraction().get());
        a().update(|v| *v = 32);
        println!("{:?}", a_b_subtraction().get());
        a().set(2);
        println!("{:?}", a_b_subtraction().get());
        a().set(25);
        println!("{:?}", a_b_subtraction().get());
        a().set(1);
        println!("{:?}", a_b_subtraction().get());

        println!("{:?}", count.get());

        assert_eq!(count.get(), 5);
    }

    #[test]
    fn test_undo() {
        a_reversible().set(3);

        a_reversible().set(5);

        b_reversible().set(10);

        a_reversible().set(4);

        assert_eq!(a_reversible().get(), 4, "We should get 4 as value for a");
        global_undo_queue().travel_backwards();
        assert_eq!(b_reversible().get(), 10, "We should get 10 as value for b");
        global_undo_queue().travel_backwards();
        assert_eq!(a_reversible().get(), 5, "We should get 5 as value for a");
        eprintln!("{:?}", a_reversible().get());
        eprintln!("{:?}", b_reversible().get());
        eprintln!("{:?}", global_undo_queue());

        global_undo_queue().travel_backwards(); // Why do we need 2 times         global_undo_queue().travel_backwards(); ?
        eprintln!("{:?}", a_reversible().get());
        eprintln!("{:?}", b_reversible().get());
        eprintln!("{:?}", global_undo_queue());
        global_undo_queue().travel_backwards();
        global_undo_queue().travel_backwards();
        assert_eq!(a_reversible().get(), 3, "We should get 3 as value for a");
        eprintln!("{:?}", a_reversible().get());
        eprintln!("{:?}", b_reversible().get());
        eprintln!("{:?}", global_undo_queue());
        global_undo_queue().travel_backwards();
        global_undo_queue().travel_backwards();
        assert_eq!(a_reversible().get(), 0, "We should get 0 as value for a");
    }

    #[test]
    fn test_update() {
        a_reversible().set(10);
        b_reversible().set(10);

        a_reversible().update(|state| *state = 45);

        assert_eq!(a_reversible().get(), 45, "We should get 45 as value for a");

        a().update(|state| *state = 40);
        assert_eq!(a().get(), 40, "We should get 40 as value for a");
    }

    #[test]
    fn test_inert_set() {
        a_reversible().inert_set(155);
        assert_eq!(a_reversible().get(), 155, "We should get 155");

        let a_b_subtraction = a_b_subtraction();
        a().set(0);
        b().set(0);

        a().inert_set(165);
        assert_eq!(
            a_b_subtraction.get(),
            0,
            "We should get 0 for subtraction because inert setting"
        );
        assert_eq!(a().get(), 165, "We should get 165");
    }

    #[test]
    fn test_delete() {
        a_reversible().delete();

        eprintln!("{:?}", a_reversible().get());

        assert_eq!(
            a_reversible().state_exists(),
            false,
            "The state  a_reversible should not exist"
        );

        a().delete();

        eprintln!("{:?}", a().get());

        assert_eq!(a().state_exists(), false, "The a state should not exist");
    }

    #[test]
    fn test_reaction() {
        let a_b_subtraction = a_b_subtraction();
        a().set(0);
        b().set(0);
        a().update(|state| *state = 40);
        assert_eq!(a().get(), 40, "We should get 40 as value for a");
        assert_eq!(
            a_b_subtraction.get(),
            40,
            "We should get 40 for subtraction because setting"
        );

        b().set(10);
        assert_eq!(
            a_b_subtraction.get(),
            30,
            "We should get 40 for subtraction because setting"
        );
        b().inert_set(0);
        assert_eq!(
            a_b_subtraction.get(),
            30,
            "We should get 30 for subtraction because setting inert"
        );
        b().set(20);
        assert_eq!(
            a_b_subtraction.get(),
            20,
            "We should get 20 for subtraction because setting"
        );
    }

    #[test]
    fn test_reversible_reaction() {
        let a_b_reversible_subtraction = a_b_reversible_subtraction();
        a_reversible().set(0);
        b_reversible().set(0);
        a_reversible().update(|state| *state = 40);
        assert_eq!(a_reversible().get(), 40, "We should get 40 as value for a");
        assert_eq!(
            a_b_reversible_subtraction.get(),
            40,
            "We should get 40 for subtraction because setting"
        );

        global_undo_queue().travel_backwards();
        assert_eq!(
            a_b_reversible_subtraction.get(),
            0,
            "We should get 0 because back in time"
        );

        b_reversible().inert_set(0);
        assert_eq!(
            a_b_reversible_subtraction.get(),
            0,
            "We should get 0 for subtraction because setting inert"
        );
        a_reversible().set(20);
        assert_eq!(
            a_b_reversible_subtraction.get(),
            20,
            "We should get 20 for subtraction because setting"
        );
    }
}
