{ writeShellScriptBin, coreutils, curl, gnugrep, gawk }:
let
  echo = "${coreutils}/bin/echo";
  pwd = "${coreutils}/bin/pwd";
  mv = "${coreutils}/bin/mv";
  curl' = "${curl}/bin/curl";
  grep' = "${gnugrep}/bin/grep";
  awk' = "${gawk}/bin/awk";
in
writeShellScriptBin "update.sh" ''
  set -euo pipefail

  latest_version=$(${curl'} -s https://www.reaper.fm/download.php | ${grep'} -oP 'REAPER \K[0-9.]+')

  function update_nix_flake() {
    local flake="$1"

    file_tag=''${latest_version%%.*} # retain only the major version
    reap_tag=''${latest_version//./} # remove the '.'
    reap_tar="reaper''${reap_tag}_linux_x86_64.tar.xz"
    reaper_tarball_link="https://www.reaper.fm/files/''${file_tag}.x/''${reap_tar}"

   ${echo} "Updating REAPER tarball URL to: ''${reaper_tarball_link}"

    ${awk'} -v new_url="$reaper_tarball_link" '
    BEGIN { url_updated = 0 }
    {
      if ($0 ~ /reaper\s*=\s*\{/ && url_updated == 0) {
        print
        while (getline) {
          if ($0 ~ /url\s*=/) {
            sub(/url\s*=\s*"[^"]*"/, "url = \"" new_url "\"")
            url_updated = 1
          }
          print
          if ($0 ~ /}/) {
            break
          }
        }
      } else {
        print
    }
  }
  ' "$flake" > "''${flake}.tmp" && ${mv} "''${flake}.tmp" "$flake"
  }

  nix_flake="$(${pwd})/flake.nix"

  if [ ! -f "$nix_flake" ]; then
    ${echo} "Error: $nix_flake does not exist."
    exit 1
  fi

  update_nix_flake "$nix_flake"
''
