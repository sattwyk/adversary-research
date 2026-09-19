;; Synchronous component fixture: short writes, nested frames, and a live canary.
(component
 (import "write" (func $write (param "offset" u32) (param "length" u32) (result u32)))
 (import "sync" (func $sync (result u32)))
 (core module $m
  (import "io" "write" (func $write (param i32 i32) (result i32)))
  (import "io" "sync" (func $sync (result i32)))
  (func $write_all (param $length i32) (result i32) (local $offset i32)
   (loop $again
    (local.set $offset (i32.add (local.get $offset)
      (call $write (local.get $offset) (i32.sub (local.get $length) (local.get $offset)))))
    (br_if $again (i32.lt_u (local.get $offset) (local.get $length))))
   (local.get $offset))
  (func $commit (param $canary i32) (result i32) (local $written i32)
   (local.set $written (call $write_all (i32.const 7)))
   (if (i32.ne (call $sync) (i32.const 0)) (then unreachable))
   (i32.add (local.get $canary) (local.get $written)))
  (func $nested (param $canary i32) (result i32)
   (call $commit (local.get $canary)))
  (func (export "run") (result i32) (call $nested (i32.const 123456))))
 (core func $w (canon lower (func $write)))
 (core func $s (canon lower (func $sync)))
 (core instance $i (instantiate $m (with "io" (instance
   (export "write" (func $w)) (export "sync" (func $s))))))
 (func (export "run") (result u32) (canon lift (core func $i "run"))))
