"use strict";

const upstream = require("brace-expansion-upstream"); // eslint-disable-line @typescript-eslint/no-require-imports

if (
  typeof upstream.expand !== "function" ||
  upstream.EXPANSION_MAX !== 100_000 ||
  upstream.EXPANSION_MAX_LENGTH !== 4_000_000
) {
  throw new TypeError(
    "brace-expansion 5.0.9 must expose the audited expansion limits",
  );
}

module.exports = upstream.expand;
module.exports.expand = upstream.expand;
module.exports.EXPANSION_MAX = upstream.EXPANSION_MAX;
module.exports.EXPANSION_MAX_LENGTH = upstream.EXPANSION_MAX_LENGTH;
