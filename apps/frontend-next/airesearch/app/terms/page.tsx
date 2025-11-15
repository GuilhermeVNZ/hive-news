import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { FileText, Scale, AlertTriangle, Shield, Mail, Book } from "lucide-react";

export default function TermsPage() {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />

      <main className="flex-grow">
        <div className="container mx-auto px-4 py-12">
          <div className="max-w-4xl mx-auto">
            <h1 className="text-4xl md:text-5xl font-bold mb-6">
              Terms of Service
            </h1>
            <p className="text-xl text-muted-foreground mb-12">
              Last updated: {new Date().toLocaleDateString("en-US", { year: "numeric", month: "long", day: "numeric" })}
            </p>

            <div className="prose prose-lg max-w-none mb-16">
              <p className="text-foreground leading-relaxed mb-6 text-justify">
                Welcome to AIResearch.news. By accessing and using this website, you agree to comply with and be bound by the following terms and conditions. Please review them carefully.
              </p>
            </div>

            {/* Section 1 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <Book className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">1. Acceptance of Terms</h2>
              </div>
              <p className="text-foreground leading-relaxed text-justify">
                By accessing and using AIResearch.news, you accept and agree to be bound by the terms and provision of this agreement. If you do not agree to abide by the above, please do not use this service.
              </p>
            </div>

            {/* Section 2 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <FileText className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">2. Use License</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                Permission is granted to temporarily access the materials on AIResearch.news for personal, non-commercial transitory viewing only. This is the grant of a license, not a transfer of title, and under this license you may not:
              </p>
              <ul className="list-disc list-inside space-y-2 text-foreground leading-relaxed text-justify ml-4">
                <li>Modify or copy the materials</li>
                <li>Use the materials for any commercial purpose or for any public display</li>
                <li>Attempt to reverse engineer any software contained on the website</li>
                <li>Remove any copyright or other proprietary notations from the materials</li>
              </ul>
            </div>

            {/* Section 3 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <AlertTriangle className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">3. Disclaimer</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                The materials on AIResearch.news are provided on an &apos;as is&apos; basis. AIResearch makes no warranties, expressed or implied, and hereby disclaims and negates all other warranties including, without limitation:
              </p>
              <ul className="list-disc list-inside space-y-2 text-foreground leading-relaxed text-justify ml-4">
                <li>Implied warranties or conditions of merchantability</li>
                <li>Fitness for a particular purpose</li>
                <li>Non-infringement of intellectual property or other violation of rights</li>
              </ul>
              <p className="text-foreground leading-relaxed mt-4 text-justify">
                AIResearch does not warrant or make any representations concerning the accuracy, likely results, or reliability of the use of the materials on its website or otherwise relating to such materials or on any sites linked to this site.
              </p>
            </div>

            {/* Section 4 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <Scale className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">4. Limitations</h2>
              </div>
              <p className="text-foreground leading-relaxed text-justify">
                In no event shall AIResearch or its suppliers be liable for any damages (including, without limitation, damages for loss of data or profit, or due to business interruption) arising out of the use or inability to use the materials on AIResearch.news, even if AIResearch or an authorized representative has been notified orally or in writing of the possibility of such damage.
              </p>
            </div>

            {/* Section 5 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <FileText className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">5. Accuracy of Materials</h2>
              </div>
              <p className="text-foreground leading-relaxed text-justify">
                The materials appearing on AIResearch.news could include technical, typographical, or photographic errors. AIResearch does not warrant that any of the materials on its website are accurate, complete, or current. AIResearch may make changes to the materials contained on its website at any time without notice.
              </p>
            </div>

            {/* Section 6 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <Shield className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">6. Intellectual Property</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                All content on AIResearch.news, including but not limited to text, graphics, logos, images, and software, is the property of AIResearch or its content suppliers and is protected by international copyright and trademark laws. You may not:
              </p>
              <ul className="list-disc list-inside space-y-2 text-foreground leading-relaxed text-justify ml-4">
                <li>Reproduce, distribute, or create derivative works from our content without permission</li>
                <li>Use our trademarks or logos without written consent</li>
                <li>Remove any copyright or proprietary notices from our materials</li>
              </ul>
            </div>

            {/* Section 7 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <FileText className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">7. Links</h2>
              </div>
              <p className="text-foreground leading-relaxed text-justify">
                AIResearch has not reviewed all of the sites linked to its website and is not responsible for the contents of any such linked site. The inclusion of any link does not imply endorsement by AIResearch of the site. Use of any such linked website is at the user&apos;s own risk.
              </p>
            </div>

            {/* Section 8 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <FileText className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">8. Modifications</h2>
              </div>
              <p className="text-foreground leading-relaxed text-justify">
                AIResearch may revise these terms of service for its website at any time without notice. By using this website you are agreeing to be bound by the then current version of these terms of service.
              </p>
            </div>

            {/* Section 9 */}
            <div className="bg-gradient-to-br from-primary/10 to-primary/5 rounded-xl p-8 border border-primary/20">
              <div className="flex items-center mb-4">
                <Mail className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">9. Contact Information</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                If you have any questions about these Terms of Service, please contact us:
              </p>
              <div className="space-y-2 text-foreground leading-relaxed">
                <p>
                  <strong>Email:</strong>{" "}
                  <a 
                    href="mailto:contact@hive-hub.ai" 
                    className="text-primary hover:underline"
                  >
                    contact@hive-hub.ai
                  </a>
                </p>
                <p>
                  <strong>Website:</strong>{" "}
                  <a 
                    href="https://www.airesearch.news" 
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline"
                  >
                    www.airesearch.news
                  </a>
                </p>
              </div>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}















