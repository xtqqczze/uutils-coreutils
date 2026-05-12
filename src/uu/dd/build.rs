// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

fn main() {
    let is_nightly =
        rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly;
    rsconf::declare_cfg("nightly", is_nightly);
}
