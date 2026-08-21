import 'package:flutter_test/flutter_test.dart';
import 'package:fnzero_safe_mobile/src/features/mobile_capabilities.dart';

void main() {
  test('mobile v1 keeps Squads and excludes Program workflows', () {
    expect(isMobileCapabilityEnabled('squads_multisig'), isTrue);
    expect(isMobileCapabilityExcluded('program_deploy'), isTrue);
    expect(isMobileCapabilityExcluded('program_upgrade'), isTrue);
    expect(isMobileCapabilityExcluded('program_invoke'), isTrue);
  });
}
