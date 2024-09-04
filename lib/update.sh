#!/usr/bin/env nix-shell
#!nix-shell -i bash -p curl common-updater-scripts

set -euo pipefail

latest_version=$(curl -s https://www.reaper.fm/download.php | grep -oP 'REAPER \K[0-9.]+')

function set_hash_for_linux() {
  file_tag=${latest_version%%.*} # retain only the major version
  reap_tag=${latest_version//./} # remove the '.'
  reap_tar="reaper${reap_tag}_linux_x86_64.tar.xz"
  reaper_tarball_link="https://www.reaper.fm/files/${file_tag}.x/${reap_tar}"

  pkg_hash=$(nix-prefetch-url "$reaper_tarball_link")
  pkg_hash=$(nix hash to-sri "sha256:$pkg_hash")

  echo ${reaper_tarball_link}
  echo ${pkg_hash}
}

set_hash_for_linux x86_64