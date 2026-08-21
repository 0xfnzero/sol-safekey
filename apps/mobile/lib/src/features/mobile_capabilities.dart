const mobileEnabledCapabilities = <String>[
  'wallet_management',
  'assets',
  'payments',
  'two_factor',
  'pump_trading',
  'dapp_signing',
  'squads_multisig',
];

const mobileExcludedCapabilities = <String>[
  'program_deploy',
  'program_upgrade',
  'program_source_build',
  'program_invoke',
];

bool isMobileCapabilityEnabled(String capability) {
  return mobileEnabledCapabilities.contains(capability);
}

bool isMobileCapabilityExcluded(String capability) {
  return mobileExcludedCapabilities.contains(capability);
}
