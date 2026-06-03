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
console.log(`Updated package.json and package-lock.json → ${version}`);

// 2. Update Cargo.toml
const cargoToml = "src-tauri/Cargo.toml";
let cargoContent = fs.readFileSync(cargoToml, "utf8");

cargoContent = cargoContent.replace(
  /^version\s*=\s*".*"/m,
  `version = "${version}"`
);

fs.writeFileSync(cargoToml, cargoContent);
console.log(`Updated ${cargoToml} → ${version}`);

// 3. Update tauri.conf.json
const tauriConfigPath = "src-tauri/tauri.conf.json";
const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath));

tauriConfig.version = version;

fs.writeFileSync(
  tauriConfigPath,
  JSON.stringify(tauriConfig, null, 2) + "\n"
);
console.log(`Updated ${tauriConfigPath} → ${version}`);

// 4. Update Cinny web submodule to latest tag
console.log("Updating cinny web submodule");

execSync("git submodule update --init --recursive", { stdio: "inherit" });

execSync("git fetch --tags", { cwd: "cinny", stdio: "inherit" });

const latestCommit = execSync("git rev-list --tags --max-count=1", {
  cwd: "cinny",
}).toString().trim();

const latestTag = execSync(`git describe --tags ${latestCommit}`, {
  cwd: "cinny",
}).toString().trim();

console.log(`Latest cinny tag: ${latestTag}`);

execSync(`git checkout ${latestTag}`, { cwd: "cinny", stdio: "inherit" });

execSync("git add cinny", { stdio: "inherit" });

console.log("Release preparation complete");