package nl.sander.sample;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class SampleTests {

    @Test
    public int getTheNumberTest() {
       assertEquals(42, Sample.getTheNumber());
    }
}