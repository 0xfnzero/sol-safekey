Pod::Spec.new do |s|
  s.name = 'FnzeroSafeMobileBridge'
  s.version = '0.1.0'
  s.summary = 'FnzeroSafe Rust mobile bridge'
  s.description = 'Rust native library for the FnzeroSafe Flutter mobile app.'
  s.homepage = 'https://github.com/0xfnzero/FnzeroSafe'
  s.license = { :type => 'MIT' }
  s.author = { 'FnzeroSafe' => 'dev@fnzero.safe' }
  s.platform = :ios, '15.0'
  s.source = { :path => '.' }
  s.vendored_frameworks = 'Frameworks/FnzeroSafeMobileBridge.xcframework'
end
