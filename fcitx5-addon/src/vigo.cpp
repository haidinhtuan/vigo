/*
 * Fcitx5 Vietnamese Input Method using vigo engine
 */

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/inputmethodengine.h>
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

        // Handle space/enter - commit
        if (key.check(FcitxKey_space) || key.check(FcitxKey_Return)) {
            if (!vigo_is_empty(engine_)) {
                commit();
                // Don't filter space - let it through
                if (key.check(FcitxKey_Return)) {
                    event.filterAndAccept();
                }
                return;
            }
            return;
        }

        // Handle escape - clear
        if (key.check(FcitxKey_Escape)) {
            if (!vigo_is_empty(engine_)) {
                reset();
                event.filterAndAccept();
                return;
            }
            return;
        }

        // Handle regular characters
        if (key.isSimple()) {
            auto chr = static_cast<uint32_t>(key.sym());
            if (chr >= 0x20 && chr < 0x7f) {
                vigo_feed(engine_, chr);
                updatePreedit();
                event.filterAndAccept();
                return;
            }
        }
    }

    void commit() {
        char *output = vigo_commit(engine_);
        if (output) {
            ic_->commitString(output);
            vigo_free_string(output);
        }
        updatePreedit();
    }

    void updatePreedit() {
        fcitx::Text preedit;
        
        char *output = vigo_get_output(engine_);
        if (output && output[0] != '\0') {
            preedit.append(output, fcitx::TextFormatFlag::Underline);
        }
        if (output) {
            vigo_free_string(output);
        }

        if (ic_->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
            ic_->inputPanel().setClientPreedit(preedit);
        } else {
            ic_->inputPanel().setPreedit(preedit);
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
    VigoEngine(fcitx::Instance *instance) : instance_(instance) {
        instance->inputContextManager().registerProperty("vigoState", &factory_);
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
    fcitx::FactoryFor<VigoState> factory_;
};

class VigoAddonFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new VigoEngine(manager->instance());
    }
};

} // namespace

FCITX_ADDON_FACTORY(VigoAddonFactory);
