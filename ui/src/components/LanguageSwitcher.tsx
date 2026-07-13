'use client';

import { useParams, usePathname, useRouter } from 'next/navigation';
import { locales, localeNames, type Locale } from '@/i18n/config';
import { Languages } from 'lucide-react';

export default function LanguageSwitcher() {
  const params = useParams();
  const locale = (params?.locale as string) || 'en';
  const router = useRouter();
  const pathname = usePathname();

  const switchLocale = (newLocale: Locale) => {
    // Replace the current locale in the pathname
    const segments = pathname.split('/');
    segments[1] = newLocale;
    const newPath = segments.join('/');
    router.push(newPath);
  };

  return (
    <div className="relative group">
      <button type="button" className="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-white/5 transition-colors">
        <Languages className="w-4 h-4" />
        <span className="text-sm">{localeNames[locale as Locale]}</span>
      </button>
      <div className="absolute right-0 mt-2 w-40 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-50">
        <div className="bg-black/90 backdrop-blur-xl rounded-lg border border-white/10 shadow-xl overflow-hidden">
          {locales.map((loc) => (
            <button type="button"
              key={loc}
              onClick={() => switchLocale(loc)}
              className={`w-full px-4 py-2 text-left text-sm hover:bg-white/10 transition-colors ${
                locale === loc ? 'bg-purple-500/20 text-purple-400' : 'text-gray-300'
              }`}
            >
              {localeNames[loc]}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
