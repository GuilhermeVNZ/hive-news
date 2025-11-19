import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { Shield, Lock, Eye, FileText, Mail, Globe } from "lucide-react";

const PrivacyPolicyPage = () => {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />

      <main className="flex-grow">
        <div className="container mx-auto px-4 py-12">
          <div className="max-w-4xl mx-auto">
            <h1 className="text-4xl md:text-5xl font-bold mb-6">
              Privacy Policy
            </h1>
            <p className="text-xl text-muted-foreground mb-12">
              Last updated: 11/04/2025
            </p>

            <div className="prose prose-lg max-w-none mb-16">
              <p className="text-foreground leading-relaxed mb-6 text-justify">
                ScienceAI respects your privacy and is committed to protecting your personal data. This policy explains what information we collect, how we use it and the choices you have regarding your data.
              </p>
            </div>

            {/* Section 1 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <FileText className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">1. Information We Collect</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                We may collect the following types of information:
              </p>
              <ul className="list-disc list-inside space-y-2 text-foreground leading-relaxed text-justify ml-4">
                <li>
                  <strong>Personal data you provide directly:</strong> such as your name and email address when subscribing to our newsletter or contacting us.
                </li>
                <li>
                  <strong>Automatically collected data:</strong> including IP address, browser type, device information, pages visited and referring website, collected through cookies or similar technologies.
                </li>
                <li>
                  <strong>Analytics data:</strong> gathered through third-party tools like Google Analytics to understand how users interact with our site.
                </li>
              </ul>
            </div>

            {/* Section 2 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <Eye className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">2. How We Use Your Information</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                We use your data to:
              </p>
              <ul className="list-disc list-inside space-y-2 text-foreground leading-relaxed text-justify ml-4">
                <li>Provide and maintain the website</li>
                <li>Send newsletters or updates when you subscribe</li>
                <li>Improve website performance and user experience</li>
                <li>Monitor website traffic, trends and user behavior</li>
                <li>Protect against unauthorized access, fraud or abuse</li>
              </ul>
              <p className="text-foreground leading-relaxed mt-4 text-justify">
                We do not sell, rent or trade your personal data.
              </p>
            </div>

            {/* Section 3 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <Shield className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">3. Cookies and Tracking Technologies</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                We use cookies and similar technologies to:
              </p>
              <ul className="list-disc list-inside space-y-2 text-foreground leading-relaxed text-justify ml-4">
                <li>Remember your preferences</li>
                <li>Analyze website traffic and performance</li>
                <li>Improve loading speed and functionality</li>
              </ul>
              <p className="text-foreground leading-relaxed mt-4 text-justify">
                You can disable cookies in your browser settings, but some features of the website may not function properly.
              </p>
            </div>

            {/* Section 4 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <Globe className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">4. Third-Party Services</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                We may use third-party providers such as:
              </p>
              <ul className="list-disc list-inside space-y-2 text-foreground leading-relaxed text-justify ml-4">
                <li><strong>Google Analytics</strong> – traffic analysis</li>
                <li><strong>Email newsletter platforms</strong> – for sending emails if you subscribe</li>
              </ul>
              <p className="text-foreground leading-relaxed mt-4 text-justify">
                These services may collect information according to their own privacy policies. We do not control how they handle your data.
              </p>
            </div>

            {/* Section 5 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <Lock className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">5. Data Storage and Security</h2>
              </div>
              <p className="text-foreground leading-relaxed text-justify">
                We take reasonable technical and organizational measures to protect your data from loss, misuse or unauthorized access. However, no method of transmission or storage is completely secure.
              </p>
            </div>

            {/* Section 6 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <Shield className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">6. Your Rights</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                Depending on your location, you may have the right to:
              </p>
              <ul className="list-disc list-inside space-y-2 text-foreground leading-relaxed text-justify ml-4">
                <li>Access the personal data we hold about you</li>
                <li>Request corrections or deletion of your data</li>
                <li>Opt out of newsletters or email communications</li>
                <li>Disable cookies or analytics tracking</li>
              </ul>
              <p className="text-foreground leading-relaxed mt-4 text-justify">
                To exercise these rights, contact us at{" "}
                <a 
                  href="mailto:contact@hive-hub.ai" 
                  className="text-primary hover:underline"
                >
                  contact@hive-hub.ai
                </a>
                .
              </p>
            </div>

            {/* Section 7 */}
            <div className="bg-card rounded-xl p-6 shadow-card mb-6">
              <div className="flex items-center mb-4">
                <FileText className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">7. Changes to this Policy</h2>
              </div>
              <p className="text-foreground leading-relaxed text-justify">
                We may update this Privacy Policy from time to time. Any changes will be posted on this page with the updated date.
              </p>
            </div>

            {/* Section 8 */}
            <div className="bg-gradient-to-br from-primary/10 to-primary/5 rounded-xl p-8 border border-primary/20">
              <div className="flex items-center mb-4">
                <Mail className="h-6 w-6 text-primary mr-3" />
                <h2 className="text-2xl font-bold">8. Contact Us</h2>
              </div>
              <p className="text-foreground leading-relaxed mb-4 text-justify">
                If you have questions about this Privacy Policy or how we handle your data, contact us:
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
                    href="https://www.scienceai.news" 
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline"
                  >
                    www.scienceai.news
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
};

export default PrivacyPolicyPage;













































