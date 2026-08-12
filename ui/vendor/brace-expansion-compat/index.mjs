import {
  EXPANSION_MAX,
  EXPANSION_MAX_LENGTH,
  expand,
} from "brace-expansion-upstream";

if (
  typeof expand !== "function" ||
  EXPANSION_MAX !== 100_000 ||
  EXPANSION_MAX_LENGTH !== 4_000_000
) {
  throw new TypeError(
    "brace-expansion 5.0.8 must expose the audited expansion limits",
  );
}

const compat = Object.assign(expand, {
  EXPANSION_MAX,
  EXPANSION_MAX_LENGTH,
  expand,
});

export { EXPANSION_MAX, EXPANSION_MAX_LENGTH, expand };
export default compat;
