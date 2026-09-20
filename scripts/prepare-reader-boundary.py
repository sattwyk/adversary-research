#!/usr/bin/env python3
"""Add one test-only checkpoint to a clean historical raft-log checkout.

Fails closed if the expected historical code differs. It never substitutes
the codec, file, reader, seek, pread, cache, or native flush worker.
"""
import pathlib
import sys

root, fixture, variant = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
chunk = root / 'src/chunk/mod.rs'
source = chunk.read_text()
anchor = ('        let br = io::BufReader::with_capacity(16 * 1024, f);' if variant == 'parent'
          else '        self.f.read_exact_at(&mut buf, offset)?;')
assert source.count(anchor) == 1, (variant, source.count(anchor))
source = source.replace(anchor, '        #[cfg(test)]\n        crate::tests::reader_boundary::checkpoint();\n' + anchor)
chunk.write_text(source)
tests = root / 'src/tests/mod.rs'
tests.write_text(tests.read_text() + '\npub(crate) mod reader_boundary;\n')
(root / 'src/tests/reader_boundary.rs').write_text(fixture.read_text())
manifest = root / 'Cargo.toml'
text = manifest.read_text()
assert text.count('[dev-dependencies]') == 1
manifest.write_text(text.replace('[dev-dependencies]', '[dev-dependencies]\nshuttle = "=0.9.3"'))
