// SPDX-License-Identifier: Apache-2.0

package com.digitalpebble.ginkgo;

import com.digitalpebble.ginkgo.model.ActionsBill;
import com.digitalpebble.ginkgo.model.UsageItem;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

import static org.junit.jupiter.api.Assertions.*;

class CarbonEstimatorTest {

    private ActionsBill loadTestBill() throws IOException {
        try (InputStream is = getClass().getClassLoader().getResourceAsStream("actions_bill.json")) {
            String json = new String(is.readAllBytes(), StandardCharsets.UTF_8);
            return ActionsBill.fromJson(json);
        }
    }

    @Test
    void setsEnergyAndCo2OnMinuteItems() throws IOException {
        ActionsBill bill = loadTestBill();
        CarbonEstimator.calculateCarbonImpact(bill);

        for (UsageItem item : bill.getUsageItems()) {
            if ("Minutes".equals(item.getUnitType())) {
                assertNotNull(item.getEnergyUsageWh(), "energyUsageWh should be set for " + item.getSku());
                assertNotNull(item.getCo2eqG(), "co2eqG should be set for " + item.getSku());
                assertTrue(item.getEnergyUsageWh() > 0);
                assertTrue(item.getCo2eqG() > 0);
            }
        }
    }

    @Test
    void skipsStorageItems() throws IOException {
        ActionsBill bill = loadTestBill();
        CarbonEstimator.calculateCarbonImpact(bill);

        for (UsageItem item : bill.getUsageItems()) {
            if ("GigabyteHours".equals(item.getUnitType())) {
                assertNull(item.getEnergyUsageWh(), "storage items should not have energyUsageWh");
                assertNull(item.getCo2eqG(), "storage items should not have co2eqG");
            }
        }
    }

    @Test
    void computesCorrectValuesForLinuxArm() throws IOException {
        ActionsBill bill = loadTestBill();
        CarbonEstimator.calculateCarbonImpact(bill);

        // "Actions Linux ARM", 29 minutes, ubuntu-arm = 45W, pue = 1.16, grid = 352
        UsageItem item = bill.getUsageItems().get(1);
        assertEquals("Actions Linux ARM", item.getSku());

        assertEquals(25.230, item.getEnergyUsageWh(), 1e-3);
        assertEquals(8.881, item.getCo2eqG(), 1e-3);
    }

    @Test
    void computesCorrectValuesForLinux() throws IOException {
        ActionsBill bill = loadTestBill();
        CarbonEstimator.calculateCarbonImpact(bill);

        // "Actions Linux", 67 minutes, ubuntu = 65W, pue = 1.16, grid = 352
        UsageItem item = bill.getUsageItems().get(2);
        assertEquals("Actions Linux", item.getSku());

        assertEquals(84.197, item.getEnergyUsageWh(), 1e-3);
        assertEquals(29.637, item.getCo2eqG(), 1e-3);
    }

    @Test
    void emptyBillProducesNoErrors() {
        ActionsBill bill = ActionsBill.fromJson("{\"usageItems\":[]}");
        CarbonEstimator.calculateCarbonImpact(bill);
        assertTrue(bill.getUsageItems().isEmpty());
    }
}
