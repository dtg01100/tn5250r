# Terminal Settings Guide

TN5250R provides comprehensive terminal configuration options through the Settings dialog. This guide explains how to use these features to optimize your terminal emulation experience.

## Accessing Terminal Settings

1. Launch TN5250R
2. Click the **Settings** button in the main menu bar
3. The Terminal Settings dialog will open

## Protocol Mode Configuration

The protocol mode determines how TN5250R communicates with your server. Choose the appropriate mode based on your target system:

### TN5250 (IBM AS/400) - Default
- **Use for**: IBM AS/400, IBM i, iSeries systems
- **Description**: Standard IBM AS/400 terminal protocol implementing RFC 2877/4777
- **Recommended**: Most AS/400 connections

### TN3270 (IBM Mainframe)
- **Use for**: IBM mainframe systems (z/OS, MVS, VM/CMS)
- **Description**: IBM mainframe terminal protocol for 3270-series terminals
- **Recommended**: Mainframe connections

### Auto-Detect
- **Use for**: Unknown or mixed environments
- **Description**: Automatically detects the appropriate protocol based on server response
- **Recommended**: When you're unsure of the server type

## Screen Size Options

Select the screen dimensions that match your server configuration and display preferences:

### Model 2 (24×80) - Default
- **Dimensions**: 24 rows × 80 columns
- **Total Characters**: 1920
- **Use for**: Standard terminal sessions, legacy applications
- **Compatible with**: Most AS/400 and mainframe systems

### Model 3 (32×80)
- **Dimensions**: 32 rows × 80 columns  
- **Total Characters**: 2560
- **Use for**: Extended display applications
- **Benefits**: More screen real estate while maintaining standard width

### Model 4 (43×80)
- **Dimensions**: 43 rows × 80 columns
- **Total Characters**: 3440
- **Use for**: Applications requiring many rows of data
- **Benefits**: Maximum vertical space for reports and listings

### Model 5 (27×132)
- **Dimensions**: 27 rows × 132 columns
- **Total Characters**: 3564
- **Use for**: Wide-format reports, spreadsheet-like applications
- **Benefits**: Accommodates wide data displays and reports

## Configuration Management

### Automatic Saving
- Settings are **automatically saved** when you make changes
- No need to manually save configuration
- Settings persist between application sessions

### Reset to Defaults
- Click **Reset to Defaults** to restore original settings:
  - Protocol Mode: TN5250 (IBM AS/400)
  - Screen Size: Model 2 (24×80)
- Useful for troubleshooting or starting fresh

### When Changes Take Effect
- Settings changes take effect on the **next connection**
- Current connections continue with previous settings
- Disconnect and reconnect to apply new settings

## Best Practices

### Choosing Protocol Mode
1. **Start with TN5250** for AS/400/IBM i systems
2. **Use TN3270** only for confirmed mainframe connections
3. **Try Auto-Detect** if you're unsure or connecting to different system types

### Selecting Screen Size
1. **Check server capabilities** - ensure your target system supports the chosen dimensions
2. **Consider your display** - larger screens benefit from bigger terminal sizes
3. **Match application requirements** - some applications expect specific screen sizes
4. **Start with Model 2** and increase if needed

### Troubleshooting
- If display appears corrupted, reset to Model 2 (24×80)
- If protocol issues occur, try Auto-Detect mode
- Check server documentation for supported terminal types
- Use Reset to Defaults to eliminate configuration issues

## Configuration File Details

Settings are stored in the TN5250R configuration file with these keys:

```json
{
  "terminal": {
    "protocolMode": "TN5250",
    "screenSize": "Model2", 
    "rows": 24,
    "cols": 80
  }
}
```

**Note**: Manual editing of the configuration file is not recommended. Use the Settings dialog for all changes.

## Advanced Usage

### Integration with Connection Settings
- Terminal settings work alongside connection settings (host, port, TLS)
- Both are saved to the same configuration file
- Settings dialog focuses on terminal behavior, connection dialog handles network settings

### Multiple Profiles
- Currently, TN5250R maintains one set of terminal settings
- Settings apply to all connections
- Consider using different TN5250R instances for different terminal configurations

## Support

If you encounter issues with terminal settings:

1. Try **Reset to Defaults** first
2. Verify your server supports the chosen protocol and screen size
3. Check the application logs for protocol negotiation errors
4. Consult your system administrator for supported terminal types

For additional help, refer to the main README.md or submit an issue on the project repository.