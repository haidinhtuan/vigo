/*
 * Fcitx5 Vietnamese Input Method using vigo engine
 *
 * Uses preedit approach (same as fcitx5-unikey). On Wayland compositors
 * like Hyprland that don't support inline client preedit, the preedit
 * text appears in a floating panel — this is standard behavior.
 */

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx-utils/i18n.h>

extern "C" {
#include "vigo.h"
}

namespace {

class VigoState : public fcitx::InputContextProperty {
public:
    VigoState(fcitx::InputContext *ic) : ic_(ic) {
        engine_ = vigo_new_telex();
    }

    ~VigoState() {
        if (engine_) {
            vigo_free(engine_);
        }
    }

    void reset() {
        if (engine_) {
            vigo_clear(engine_);
        }
        updatePreedit();
    }

    void keyEvent(fcitx::KeyEvent &event) {
        auto key = event.key();

        // Let modifier combinations pass through (for hotkeys)
        if (key.states().test(fcitx::KeyState::Ctrl) ||
            key.states().test(fcitx::KeyState::Alt) ||
            key.states().test(fcitx::KeyState::Super)) {
            return;
        }

        // Handle backspace
        if (key.check(FcitxKey_BackSpace)) {
            if (!vigo_is_empty(engine_)) {
                vigo_backspace(engine_);
                updatePreedit();
                event.filterAndAccept();
                return;
            }
            return;
        }

        // Handle space - commit current word and pass space through
        if (key.check(FcitxKey_space)) {
            if (!vigo_is_empty(engine_)) {
                commit();
            }
            return;
        }

        // Handle enter - commit and pass through
        if (key.check(FcitxKey_Return)) {
            if (!vigo_is_empty(engine_)) {
                commit();
            }
            return;
        }

        // Handle escape - clear buffer
        if (key.check(FcitxKey_Escape)) {
            if (!vigo_is_empty(engine_)) {
                reset();
                event.filterAndAccept();
                return;
            }
            return;
        }

        // Handle regular characters
        auto sym = key.sym();
        uint32_t chr = 0;

        if (sym >= FcitxKey_A && sym <= FcitxKey_Z) {
            chr = sym;
        } else if (sym >= FcitxKey_a && sym <= FcitxKey_z) {
            if (key.states().test(fcitx::KeyState::Shift)) {
                chr = sym - FcitxKey_a + FcitxKey_A;
            } else {
                chr = sym;
            }
        } else if (sym >= FcitxKey_0 && sym <= FcitxKey_9) {
            chr = sym;
        } else if (sym >= 0x20 && sym < 0x7f) {
            // Punctuation - commit current buffer first, then pass through
            if (!vigo_is_empty(engine_)) {
                commit();
            }
            return;
        }

        if (chr != 0) {
            vigo_feed(engine_, chr);
            updatePreedit();
            event.filterAndAccept();
            return;
        }
    }

    void commit() {
        char *output = vigo_commit(engine_);
        if (output && output[0] != '\0') {
            // Clear preedit BEFORE committing to avoid duplicate text
            auto &panel = ic_->inputPanel();
            panel.setClientPreedit(fcitx::Text());
            panel.setPreedit(fcitx::Text());
            ic_->updatePreedit();

            ic_->commitString(output);
        }
        if (output) {
            vigo_free_string(output);
        }
        ic_->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    }

    void updatePreedit() {
        auto &panel = ic_->inputPanel();
        panel.reset();

        fcitx::Text preedit;
        char *output = vigo_get_output(engine_);
        if (output && output[0] != '\0') {
            bool useClient = ic_->capabilityFlags().test(
                fcitx::CapabilityFlag::Preedit);
            preedit.append(output,
                useClient ? fcitx::TextFormatFlag::Underline
                          : fcitx::TextFormatFlag::NoFlag);
            preedit.setCursor(preedit.textLength());
            if (useClient) {
                panel.setClientPreedit(preedit);
            } else {
                panel.setPreedit(preedit);
            }
        }
        if (output) {
            vigo_free_string(output);
        }

        ic_->updatePreedit();
        ic_->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    }

private:
    fcitx::InputContext *ic_;
    vigo_engine_t *engine_ = nullptr;
};

class VigoEngine : public fcitx::InputMethodEngineV2 {
public:
    VigoEngine(fcitx::Instance *instance) : instance_(instance),
        factory_([](fcitx::InputContext &ic) { return new VigoState(&ic); }) {
        instance->inputContextManager().registerProperty("vigoState", &factory_);
    }

    std::vector<fcitx::InputMethodEntry> listInputMethods() override {
        std::vector<fcitx::InputMethodEntry> result;
        result.emplace_back("vigo", "Vietnamese (Vigo Telex)", "vi", "vigo");
        result.back().setIcon("vigo").setLabel("VI");
        return result;
    }

    void activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        auto *state = event.inputContext()->propertyFor(&factory_);
        state->reset();
    }

    void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override {
        auto *state = event.inputContext()->propertyFor(&factory_);
        state->commit();
        reset(entry, event);
    }

    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        auto *state = event.inputContext()->propertyFor(&factory_);
        state->reset();
    }

    void keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &event) override {
        if (event.isRelease()) {
            return;
        }
        auto *state = event.inputContext()->propertyFor(&factory_);
        state->keyEvent(event);
    }

private:
    fcitx::Instance *instance_;
    fcitx::LambdaInputContextPropertyFactory<VigoState> factory_;
};

class VigoAddonFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new VigoEngine(manager->instance());
    }
};

} // namespace

FCITX_ADDON_FACTORY(VigoAddonFactory);
