import { ReaddyLandingBenefits } from "@/components/landing/readdy/ReaddyLandingBenefits";
import { ReaddyLandingContact } from "@/components/landing/readdy/ReaddyLandingContact";
import { ReaddyLandingFeatures } from "@/components/landing/readdy/ReaddyLandingFeatures";
import { ReaddyLandingFooter } from "@/components/landing/readdy/ReaddyLandingFooter";
import { ReaddyLandingHero } from "@/components/landing/readdy/ReaddyLandingHero";
import { ReaddyLandingHowItWorks } from "@/components/landing/readdy/ReaddyLandingHowItWorks";
import { ReaddyLandingNavbar } from "@/components/landing/readdy/ReaddyLandingNavbar";

/**
 * Marketing home — structure mirrors Readdy preview `pages/home/page.tsx`
 * (Navbar → sections → Footer inside `<main>`).
 */
export default function Home() {
  return (
    <div className="min-h-screen bg-[#0A0D0F] text-white antialiased selection:bg-[#00E5A0]/30">
      <main id="main-content">
        <ReaddyLandingNavbar />
        <ReaddyLandingHero />
        <ReaddyLandingHowItWorks />
        <ReaddyLandingFeatures />
        <ReaddyLandingBenefits />
        <ReaddyLandingContact />
        <ReaddyLandingFooter />
      </main>
    </div>
  );
}
