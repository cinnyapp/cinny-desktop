import fs from "fs";
import path from "path";
import { execSync } from "child_process";

const version = process.argv[2];

if (!version) {
  console.error("Missing version");
  process.exit(1);
}

console.log(`Preparing release ${version}`);

// 1. Update npm versions
execSync(`npm version ${version} --no-git-tag-version`, {
  stdio: "inherit"
});

// 2. Update Cargo.toml
const cargoToml = "src-tauri/Cargo.toml";
let cargoContent = fs.readFileSync(cargoToml, "utf8");

cargoContent = cargoContent.replace(
  /^version\s*=\s*".*"/m,
  `version = "${version}"`
);

fs.writeFileSync(cargoToml, cargoContent);

// 3. Update Cargo.lock
const cargoLock = "src-tauri/Cargo.lock";
let lockContent = fs.readFileSync(cargoLock, "utf8");

lockContent = lockContent.replace(
  /^version\s*=\s*".*"/m,
  `version = "${version}"`
);

fs.writeFileSync(cargoLock, lockContent);

// 4. Update tauri.conf.json
const tauriConfigPath = "src-tauri/tauri.conf.json";
const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath));

tauriConfig.package.version = version;

fs.writeFileSync(
  tauriConfigPath,
  JSON.stringify(tauriConfig, null, 2) + "\n"
);

// 5. Update Cinny web submodule to latest tag
console.log("Updating cinny web submodule");

execSync("git submodule update --init --recursive", { stdio: "inherit" });

execSync("cd cinny && git fetch --tags", { stdio: "inherit" });

const latestTag = execSync(
  "cd cinny && git describe --tags $(git rev-list --tags --max-count=1)"
)
  .toString()
  .trim();

console.log(`Latest cinny tag: ${latestTag}`);

execSync(`cd cinny && git checkout ${latestTag}`, { stdio: "inherit" });

execSync("git add cinny", { stdio: "inherit" });

console.log("Release preparation complete");