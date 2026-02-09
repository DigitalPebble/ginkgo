// SPDX-License-Identifier: Apache-2.0

package com.digitalpebble.ginkgo;

import org.junit.jupiter.api.Test;

import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;

class ConfigTest {

    @Test
    void loadsSuccessfully() {
        Config config = Config.getInstance();
        assertNotNull(config);
    }

    @Test
    void gridCarbonIntensity() {
        assertEquals(352, Config.getInstance().getGridCarbonIntensity(), 1e-9);
    }

    @Test
    void pue() {
        assertEquals(1.16, Config.getInstance().getPue(), 1e-9);
    }

    @Test
    void runnerPowerConsumptionContainsAllRunners() {
        Map<String, Double> runners = Config.getInstance().getRunnerPowerConsumption();
        assertEquals(6, runners.size());
        assertEquals(65, runners.get("ubuntu"), 1e-9);
        assertEquals(45, runners.get("ubuntu-arm"), 1e-9);
        assertEquals(75, runners.get("windows"), 1e-9);
        assertEquals(55, runners.get("windows-arm"), 1e-9);
        assertEquals(40, runners.get("macos"), 1e-9);
        assertEquals(85, runners.get("macos-13"), 1e-9);
    }
}
