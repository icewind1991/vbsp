use crate::*;
use binread::io::Cursor;
use binread::BinReaderExt;
use std::borrow::Cow;

pub struct BspFile<'a> {
    data: &'a [u8],
    directories: Directories,
    header: Header,
}

impl<'a> BspFile<'a> {
    pub fn new(data: &'a [u8]) -> BspResult<Self> {
        const EXPECTED_HEADER: Header = Header {
            v: b'V',
            b: b'B',
            s: b'S',
            p: b'P',
        };
        // TODO: Use this to decide on the version to parse it as
        const EXPECTED_VERSION: u32 = 0x14;

        let mut cursor = Cursor::new(data);
        let header: Header = cursor.read_le()?;
        let version: u32 = cursor.read_le()?;

        if header != EXPECTED_HEADER || version != EXPECTED_VERSION {
            return Err(BspError::UnexpectedHeader(header));
        }

        let directories = cursor.read_le()?;

        let map_version: u32 = cursor.read_le()?;
        dbg!(map_version);

        Ok(BspFile {
            data,
            directories,
            header,
        })
    }

    pub fn directories(&self) -> &Directories {
        &self.directories
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn lump_reader(&self, lump: LumpType) -> BspResult<LumpReader<Cursor<Cow<[u8]>>>> {
        let data = self.get_lump(lump)?;
        Ok(LumpReader::new(data))
    }

    fn get_lump(&self, lump: LumpType) -> BspResult<Cow<[u8]>> {
        let lump = &self.directories[lump];
        let raw_data = self
            .data
            .get(lump.offset as usize..lump.offset as usize + lump.length as usize)
            .ok_or_else(|| BspError::LumpOutOfBounds(lump.clone()))?;

        Ok(match lump.ident {
            0 => Cow::Borrowed(raw_data),
            _ => {
                let mut data: Vec<u8> = Vec::with_capacity(lump.ident as usize);
                let mut cursor = Cursor::new(raw_data);
                lzma_rs::lzma_decompress(&mut cursor, &mut data)
                    .map_err(BspError::LumpDecompressError)?;
                if data.len() != lump.ident as usize {
                    return Err(BspError::UnexpectedUncompressedLumpSize {
                        got: data.len() as u32,
                        expected: lump.ident,
                    });
                }

                Cow::Owned(data)
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LumpType {
    Entities,
    Planes,
    TextureData,
    Vertices,
    Visibility,
    Nodes,
    TextureInfo,
    Faces,
    Lighting,
    Occlusion,
    Leaves,
    FaceIds,
    Edges,
    SurfaceEdges,
    Models,
    WorldLights,
    LeafFaces,
    LeafBrushes,
    Brushes,
    BrushSides,
    Areas,
    AreaPortals,
    Unused0,
    Unused1,
    Unused2,
    Unused3,
    DisplayInfo,
    OriginalFaces,
    PhysDisplay,
    PhysCollide,
    VertNormals,
    VertNormalIndices,
    DisplayLightMapAlphas,
    DisplayVertices,
    DisplayLightMapSamplePositions,
    GameLump,
    LeafWaterData,
    Primitives,
    PrimVertices,
    PrimIndices,
    PakFile,
    ClipPortalVertices,
    CubeMaps,
    TextureDataStringData,
    TextureDataStringTable,
    Overlays,
    LeafMinimumDistanceToWater,
    FaceMacroTextureInfo,
    DisplayTris,
    PhysicsCollideSurface,
    WaterOverlays,
    LeafAmbientIndexHdr,
    LeafAmbientIndex,
    LightingHdr,
    WorldLightsHdr,
    LeafAmbientLightingHdr,
    LeafAmbientLighting,
    XZipPakFile,
    FacesHdr,
    MapFlags,
    OverlayFades,
    OverlaySystemLevels,
    PhysLevel,
    DisplayMultiBlend,
}
