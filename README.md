Showing how adding an `impl` with `specialization` in a dependency can change existing code to mean something different, all while considered by conventional wisdom to be a minor revision of that dependency.

```
> cargo run
Inoccuous delegation from derived display impl
> cargo run --features trait_impl/with-specialization # Enables an impl in a dependency
Hostile takeover from new ShowDetails impl
```

Compare that to the alternative:
```
> cargo run --features trait_host/with-alternative
error[E0277]: the trait bound `FizzBuzzer: ShowDetails` is not satisfied
> cargo run --features trait_impl/with-alternative
Inoccuous delegation from derived display impl
> cargo run --features trait_impl/with-alternative,trait_impl/with-override
Normal takeover from new ShowDetails impl
```
